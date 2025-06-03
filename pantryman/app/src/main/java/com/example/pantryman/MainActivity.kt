package com.example.pantryman

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.widget.*
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import androidx.appcompat.app.AlertDialog
import android.view.LayoutInflater
import android.util.Log
import android.content.Intent
import android.content.SharedPreferences
import android.app.Activity
import com.google.android.material.floatingactionbutton.FloatingActionButton

class MainActivity : AppCompatActivity() {
    private lateinit var cookbookEngine: CookbookEngine
    private lateinit var recyclerView: RecyclerView
    private lateinit var adapter: IngredientsAdapter
    private lateinit var categorySpinner: Spinner
    private lateinit var addButton: FloatingActionButton
    private lateinit var settingsButton: Button
    private lateinit var statusText: TextView
    private lateinit var sharedPreferences: SharedPreferences
    
    private var allIngredients = listOf<Ingredient>()
    private var categories = listOf<String>()

    companion object {
        private const val PREFS_NAME = "PantrymanPrefs"
        private const val PREF_DATA_DIR = "data_directory"
        private const val DEFAULT_DATA_DIR = "cookbook_data" // relative to internal storage
        private const val REQUEST_CODE_SETTINGS = 2001
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        // Hide the action bar to prevent overlap with buttons
        supportActionBar?.hide()
        
        Log.d("MainActivity", "=== PANTRYMAN APP STARTING ===")
        Log.d("MainActivity", "onCreate called")
        
        sharedPreferences = getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
        
        initializeViews()
        initializeCookbookEngine()
        setupCategorySpinner()
        setupRecyclerView()
        setupAddButton()
        setupSettingsButton()
        loadIngredients()
        
        Log.d("MainActivity", "=== PANTRYMAN APP STARTUP COMPLETE ===")
    }

    override fun onResume() {
        super.onResume()
        Log.d("MainActivity", "onResume called")
        
        // Check if data directory has changed and reinitialize if necessary
        if (::cookbookEngine.isInitialized) {
            handleDataDirectoryChange()
        } else {
            Log.d("MainActivity", "Engine not initialized, skipping reload")
        }
    }

    private fun initializeViews() {
        Log.d("MainActivity", "initializeViews() called")
        recyclerView = findViewById(R.id.recyclerViewIngredients)
        categorySpinner = findViewById(R.id.spinnerCategory)
        addButton = findViewById(R.id.buttonAdd)
        settingsButton = findViewById(R.id.buttonSettings)
        statusText = findViewById(R.id.statusText)
        Log.d("MainActivity", "Views initialized successfully")
    }

    private fun initializeCookbookEngine() {
        try {
            // Get data directory from preferences or use default
            val dataPath = getDataDirectory()
            
            // Create data directory if it doesn't exist
            val dataDir = java.io.File(dataPath)
            if (!dataDir.exists()) {
                dataDir.mkdirs()
                Log.d("MainActivity", "Created data directory: $dataPath")
            }
            
            // Copy bundled assets to data directory if it's empty
            setupInitialData(dataDir)
            
            cookbookEngine = CookbookEngine(dataPath)
            Log.d("MainActivity", "CookbookEngine initialized successfully with path: $dataPath")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to initialize CookbookEngine", e)
            showError("Failed to initialize cookbook engine: ${e.message}")
        }
    }
    
    private fun getDataDirectory(): String {
        val savedPath = sharedPreferences.getString(PREF_DATA_DIR, null)
        return if (savedPath != null) {
            // Handle both file:// URIs and regular paths
            if (savedPath.startsWith("content://")) {
                // For now, fall back to default if it's a content URI
                // TODO: Implement proper content URI to file path conversion
                Log.w("MainActivity", "Content URI data directories not yet fully supported, using default")
                "${filesDir.absolutePath}/$DEFAULT_DATA_DIR"
            } else {
                savedPath
            }
        } else {
            "${filesDir.absolutePath}/$DEFAULT_DATA_DIR"
        }
    }
    
    private fun setupInitialData(dataDir: java.io.File) {
        try {
            // For debugging: always force a fresh copy to ensure consistency
            // TODO: In production, you might want to make this conditional or add a version check
            Log.d("MainActivity", "Setting up initial data from assets (force refresh)...")
            
            // Clear existing data directory
            if (dataDir.exists()) {
                dataDir.deleteRecursively()
            }
            
            // Create directory structure
            val ingredientsDir = java.io.File(dataDir, "ingredients")
            val recipesDir = java.io.File(dataDir, "recipes")
            ingredientsDir.mkdirs()
            recipesDir.mkdirs()
            
            // Copy pantry.yaml
            val pantryFile = java.io.File(dataDir, "pantry.yaml")
            copyAssetToFile("pantry.yaml", pantryFile)
            
            // Copy ingredients
            val assetIngredients = assets.list("ingredients") ?: emptyArray()
            for (ingredient in assetIngredients) {
                val destFile = java.io.File(ingredientsDir, ingredient)
                copyAssetToFile("ingredients/$ingredient", destFile)
            }
            
            // Copy recipes if any exist
            val assetRecipes = assets.list("recipes") ?: emptyArray()
            for (recipe in assetRecipes) {
                // Skip directories (like img/)
                if (!recipe.endsWith("/") && !recipe.equals("img")) {
                    val destFile = java.io.File(recipesDir, recipe)
                    copyAssetToFile("recipes/$recipe", destFile)
                }
            }
            
            Log.d("MainActivity", "Initial data setup complete")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to setup initial data", e)
        }
    }
    
    private fun copyAssetToFile(assetPath: String, destFile: java.io.File) {
        try {
            assets.open(assetPath).use { inputStream ->
                destFile.outputStream().use { outputStream ->
                    inputStream.copyTo(outputStream)
                }
            }
            Log.d("MainActivity", "Copied asset: $assetPath -> ${destFile.absolutePath}")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to copy asset $assetPath", e)
        }
    }

    private fun setupCategorySpinner() {
        categorySpinner.onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(parent: AdapterView<*>, view: android.view.View?, position: Int, id: Long) {
                val selectedCategory = if (position == 0) null else categories[position - 1]
                filterByCategory(selectedCategory)
            }

            override fun onNothingSelected(parent: AdapterView<*>) {}
        }
    }

    private fun setupRecyclerView() {
        adapter = IngredientsAdapter(
            onIngredientClick = { ingredient ->
                showIngredientDetails(ingredient)
            },
            onPantryStatusChange = { ingredient, isInPantry ->
                updatePantryStatus(ingredient, isInPantry)
            }
        )
        
        recyclerView.layoutManager = LinearLayoutManager(this)
        recyclerView.adapter = adapter
    }

    private fun setupAddButton() {
        Log.d("MainActivity", "Setting up add button click listener")
        Log.d("MainActivity", "Add button isClickable: ${addButton.isClickable}")
        Log.d("MainActivity", "Add button isFocusable: ${addButton.isFocusable}")
        Log.d("MainActivity", "Add button isEnabled: ${addButton.isEnabled}")
        
        addButton.setOnClickListener {
            Log.w("MainActivity", "*** ADD BUTTON CLICKED ***")
            showAddIngredientDialog()
        }
        
        // Also try setting an onTouchListener for debugging
        addButton.setOnTouchListener { view, event ->
            Log.w("MainActivity", "*** ADD BUTTON TOUCHED *** - action: ${event.action}")
            false // Return false to allow normal click handling
        }
    }

    private fun setupSettingsButton() {
        Log.d("MainActivity", "Setting up settings button click listener")
        Log.d("MainActivity", "Settings button isClickable: ${settingsButton.isClickable}")
        Log.d("MainActivity", "Settings button isFocusable: ${settingsButton.isFocusable}")
        Log.d("MainActivity", "Settings button isEnabled: ${settingsButton.isEnabled}")
        
        settingsButton.setOnClickListener {
            Log.w("MainActivity", "*** SETTINGS BUTTON CLICKED ***")
            val intent = Intent(this, SettingsActivity::class.java)
            startActivityForResult(intent, REQUEST_CODE_SETTINGS)
        }
        
        // Also try setting an onTouchListener for debugging
        settingsButton.setOnTouchListener { view, event ->
            Log.w("MainActivity", "*** SETTINGS BUTTON TOUCHED *** - action: ${event.action}")
            false // Return false to allow normal click handling
        }
    }

    private fun updateStatusText(message: String?, visible: Boolean = true) {
        statusText.visibility = if (visible) android.view.View.VISIBLE else android.view.View.GONE
        if (message != null) {
            statusText.text = message
        }
    }

    private fun loadIngredients() {
        Log.d("MainActivity", "loadIngredients() called")
        updateStatusText("Loading ingredients...", true)
        
        try {
            allIngredients = cookbookEngine.getAllIngredients()
            Log.d("MainActivity", "Loaded ${allIngredients.size} ingredients")
            
            // Extract unique categories
            categories = allIngredients.map { it.category }.distinct().sorted()
            Log.d("MainActivity", "Found ${categories.size} categories: $categories")
            
            // Setup category spinner
            val spinnerItems = listOf("All Categories") + categories
            val spinnerAdapter = ArrayAdapter(this, android.R.layout.simple_spinner_item, spinnerItems)
            spinnerAdapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item)
            categorySpinner.adapter = spinnerAdapter
            
            // Show all ingredients initially
            filterByCategory(null)
            
            // Hide loading text when successful
            updateStatusText(null, false)
            
            Log.d("MainActivity", "Ingredients loaded and displayed successfully")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to load ingredients", e)
            updateStatusText("Failed to load ingredients: ${e.message}", true)
            showError("Failed to load ingredients: ${e.message}")
        }
    }

    private fun filterByCategory(category: String?) {
        val filteredIngredients = if (category == null) {
            allIngredients
        } else {
            allIngredients.filter { it.category == category }
        }
        
        adapter.updateIngredients(filteredIngredients)
    }

    private fun showEditPantryDialog(ingredient: Ingredient) {
        val dialogView = LayoutInflater.from(this).inflate(R.layout.dialog_edit_pantry, null)
        val ingredientNameText = dialogView.findViewById<TextView>(R.id.textViewIngredientName)
        val quantityEdit = dialogView.findViewById<EditText>(R.id.editTextQuantity)
        val quantityTypeSpinner = dialogView.findViewById<Spinner>(R.id.spinnerQuantityType)
        val inPantryCheckbox = dialogView.findViewById<CheckBox>(R.id.checkBoxInPantry)

        // Set ingredient name
        ingredientNameText.text = ingredient.name

        // Set up quantity type spinner
        val quantityTypes = listOf("", "kg", "g", "lb", "oz", "pieces", "cups", "tbsp", "tsp", "ml", "l", "fl oz")
        val spinnerAdapter = ArrayAdapter(this, android.R.layout.simple_spinner_item, quantityTypes)
        spinnerAdapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item)
        quantityTypeSpinner.adapter = spinnerAdapter

        // Pre-fill with current values
        inPantryCheckbox.isChecked = ingredient.isInPantry
        ingredient.quantity?.let { quantityEdit.setText(it) }
        ingredient.quantityType?.let { currentType ->
            val index = quantityTypes.indexOf(currentType)
            if (index >= 0) {
                quantityTypeSpinner.setSelection(index)
            }
        }

        // Enable/disable quantity fields based on pantry status
        val enableQuantityFields = { enabled: Boolean ->
            quantityEdit.isEnabled = enabled
            quantityTypeSpinner.isEnabled = enabled
            if (!enabled) {
                quantityEdit.setText("")
                quantityTypeSpinner.setSelection(0)
            }
        }

        enableQuantityFields(ingredient.isInPantry)
        
        inPantryCheckbox.setOnCheckedChangeListener { _, isChecked ->
            enableQuantityFields(isChecked)
        }

        AlertDialog.Builder(this)
            .setTitle("Edit Pantry Status")
            .setView(dialogView)
            .setPositiveButton("Save") { _, _ ->
                val isInPantry = inPantryCheckbox.isChecked
                val quantity = if (isInPantry && quantityEdit.text.toString().isNotEmpty()) {
                    quantityEdit.text.toString().toDoubleOrNull()
                } else null
                val quantityType = if (isInPantry && quantityTypeSpinner.selectedItemPosition > 0) {
                    quantityTypes[quantityTypeSpinner.selectedItemPosition]
                } else null
                
                updatePantryStatus(ingredient, isInPantry, quantity, quantityType)
            }
            .setNegativeButton("Cancel") { dialog, _ -> dialog.dismiss() }
            .show()
    }

    private fun updatePantryStatus(ingredient: Ingredient, isInPantry: Boolean, quantity: Double? = null, quantityType: String? = null) {
        try {
            cookbookEngine.updatePantryStatus(ingredient.name, isInPantry, quantity, quantityType)
            
            // Update local data
            val updatedIngredients = allIngredients.map { 
                if (it.name == ingredient.name) {
                    it.copy(isInPantry = isInPantry)
                } else {
                    it
                }
            }
            allIngredients = updatedIngredients
            
            // Refresh the current view
            filterByCategory(if (categorySpinner.selectedItemPosition == 0) null else categories[categorySpinner.selectedItemPosition - 1])
            
            val action = if (isInPantry) "added to" else "removed from"
            Toast.makeText(this, "${ingredient.name} $action pantry", Toast.LENGTH_SHORT).show()
            
            Log.d("MainActivity", "Updated pantry status for ${ingredient.name}: $isInPantry")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to update pantry status", e)
            showError("Failed to update pantry: ${e.message}")
        }
    }

    // Simplified version for checkbox toggles
    private fun updatePantryStatus(ingredient: Ingredient, isInPantry: Boolean) {
        updatePantryStatus(ingredient, isInPantry, null, null)
    }

    private fun showIngredientDetails(ingredient: Ingredient) {
        AlertDialog.Builder(this)
            .setTitle(ingredient.name)
            .setMessage(buildString {
                append("Category: ${ingredient.category}\n")
                if (ingredient.tags.isNotEmpty()) {
                    append("Tags: ${ingredient.tags.joinToString(", ")}\n")
                }
                if (ingredient.isInPantry) {
                    append("Status: In pantry\n")
                    ingredient.quantity?.let { append("Quantity: $it\n") }
                    ingredient.quantityType?.let { append("Unit: $it\n") }
                    ingredient.lastUpdated?.let { append("Last updated: $it") }
                } else {
                    append("Status: Not in pantry")
                }
            })
            .setPositiveButton("OK") { dialog, _ -> dialog.dismiss() }
            .setNeutralButton("Edit") { _, _ -> showEditIngredientDialog(ingredient) }
            .setNegativeButton("Edit Pantry") { _, _ -> showEditPantryDialog(ingredient) }
            .show()
    }

    private fun showAddIngredientDialog() {
        val dialogView = LayoutInflater.from(this).inflate(R.layout.dialog_add_ingredient, null)
        val nameEdit = dialogView.findViewById<EditText>(R.id.editTextName)
        val categoryEdit = dialogView.findViewById<EditText>(R.id.editTextCategory)
        val tagsEdit = dialogView.findViewById<EditText>(R.id.editTextTags)

        AlertDialog.Builder(this)
            .setTitle("Add New Ingredient")
            .setView(dialogView)
            .setPositiveButton("Add") { _, _ ->
                val name = nameEdit.text.toString().trim()
                val category = categoryEdit.text.toString().trim()
                val tags = tagsEdit.text.toString().split(",").map { it.trim() }.filter { it.isNotEmpty() }
                
                if (name.isNotEmpty() && category.isNotEmpty()) {
                    addNewIngredient(name, category, tags)
                } else {
                    showError("Name and category are required")
                }
            }
            .setNegativeButton("Cancel") { dialog, _ -> dialog.dismiss() }
            .show()
    }

    private fun showEditIngredientDialog(ingredient: Ingredient) {
        val dialogView = LayoutInflater.from(this).inflate(R.layout.dialog_add_ingredient, null)
        val nameEdit = dialogView.findViewById<EditText>(R.id.editTextName)
        val categoryEdit = dialogView.findViewById<EditText>(R.id.editTextCategory)
        val tagsEdit = dialogView.findViewById<EditText>(R.id.editTextTags)

        // Pre-fill with current values
        nameEdit.setText(ingredient.name)
        categoryEdit.setText(ingredient.category)
        tagsEdit.setText(ingredient.tags.joinToString(", "))

        AlertDialog.Builder(this)
            .setTitle("Edit Ingredient")
            .setView(dialogView)
            .setPositiveButton("Save") { _, _ ->
                val name = nameEdit.text.toString().trim()
                val category = categoryEdit.text.toString().trim()
                val tags = tagsEdit.text.toString().split(",").map { it.trim() }.filter { it.isNotEmpty() }
                
                if (name.isNotEmpty() && category.isNotEmpty()) {
                    updateIngredient(ingredient.name, name, category, tags)
                } else {
                    showError("Name and category are required")
                }
            }
            .setNegativeButton("Cancel") { dialog, _ -> dialog.dismiss() }
            .show()
    }

    private fun addNewIngredient(name: String, category: String, tags: List<String>) {
        try {
            cookbookEngine.createIngredient(name, category, null, tags)
            loadIngredients() // Reload to get the new ingredient
            Toast.makeText(this, "Ingredient '$name' added successfully", Toast.LENGTH_SHORT).show()
            Log.d("MainActivity", "Added new ingredient: $name")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to add ingredient", e)
            showError("Failed to add ingredient: ${e.message}")
        }
    }

    private fun updateIngredient(oldName: String, newName: String, category: String, tags: List<String>) {
        try {
            cookbookEngine.updateIngredient(oldName, newName, category, null, tags)
            loadIngredients() // Reload to get updated data
            Toast.makeText(this, "Ingredient updated successfully", Toast.LENGTH_SHORT).show()
            Log.d("MainActivity", "Updated ingredient: $oldName -> $newName")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to update ingredient", e)
            showError("Failed to update ingredient: ${e.message}")
        }
    }

    private fun showError(message: String) {
        AlertDialog.Builder(this)
            .setTitle("Error")
            .setMessage(message)
            .setPositiveButton("OK") { dialog, _ -> dialog.dismiss() }
            .show()
        
        Toast.makeText(this, message, Toast.LENGTH_LONG).show()
    }

    /**
     * Reinitialize the cookbook engine with a new data directory
     * This allows for dynamic data directory switching without app restart
     */
    fun reinitializeWithNewDataDirectory(newDataPath: String): Boolean {
        return try {
            Log.d("MainActivity", "Reinitializing cookbook engine with new data directory: $newDataPath")
            updateStatusText("Switching to new data directory...", true)
            
            // Create new data directory if it doesn't exist
            val dataDir = java.io.File(newDataPath)
            if (!dataDir.exists()) {
                dataDir.mkdirs()
                Log.d("MainActivity", "Created new data directory: $newDataPath")
            }
            
            // Initialize new cookbook engine
            val newEngine = CookbookEngine(newDataPath)
            
            // If successful, replace the old engine
            cookbookEngine = newEngine
            Log.d("MainActivity", "Successfully switched to new data directory")
            
            // Reload ingredients with new engine
            loadIngredients()
            
            updateStatusText("Switched to new data directory successfully", true)
            // Hide status after a delay
            statusText.postDelayed({
                updateStatusText(null, false)
            }, 2000)
            
            true
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to reinitialize with new data directory", e)
            updateStatusText("Failed to switch data directory: ${e.message}", true)
            false
        }
    }
    
    /**
     * Handle data directory change from settings
     * Called when user selects a new data directory in SettingsActivity
     */
    fun handleDataDirectoryChange() {
        Log.d("MainActivity", "Handling data directory change")
        val newDataPath = getDataDirectory()
        reinitializeWithNewDataDirectory(newDataPath)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        
        if (requestCode == REQUEST_CODE_SETTINGS && resultCode == Activity.RESULT_OK) {
            val dataDirectoryChanged = data?.getBooleanExtra("data_directory_changed", false) ?: false
            if (dataDirectoryChanged) {
                Log.d("MainActivity", "Data directory changed, reinitializing engine")
                handleDataDirectoryChange()
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (::cookbookEngine.isInitialized) {
            cookbookEngine.cleanup()
        }
    }
}
