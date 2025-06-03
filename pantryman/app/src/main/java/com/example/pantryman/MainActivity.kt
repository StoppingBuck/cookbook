package com.example.pantryman

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.widget.*
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import androidx.appcompat.app.AlertDialog
import android.view.LayoutInflater
import android.util.Log
import com.google.android.material.floatingactionbutton.FloatingActionButton

class MainActivity : AppCompatActivity() {
    private lateinit var cookbookEngine: CookbookEngine
    private lateinit var recyclerView: RecyclerView
    private lateinit var adapter: IngredientsAdapter
    private lateinit var categorySpinner: Spinner
    private lateinit var addButton: FloatingActionButton
    
    private var allIngredients = listOf<Ingredient>()
    private var categories = listOf<String>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        Log.d("MainActivity", "=== PANTRYMAN APP STARTING ===")
        Log.d("MainActivity", "onCreate called")
        
        initializeViews()
        initializeCookbookEngine()
        setupCategorySpinner()
        setupRecyclerView()
        setupAddButton()
        loadIngredients()
        
        Log.d("MainActivity", "=== PANTRYMAN APP STARTUP COMPLETE ===")
    }

    private fun initializeViews() {
        Log.d("MainActivity", "initializeViews() called")
        recyclerView = findViewById(R.id.recyclerViewIngredients)
        categorySpinner = findViewById(R.id.spinnerCategory)
        addButton = findViewById(R.id.buttonAdd)
        Log.d("MainActivity", "Views initialized successfully")
    }

    private fun initializeCookbookEngine() {
        try {
            // Use app's internal storage for data directory
            val dataPath = filesDir.absolutePath + "/cookbook_data"
            
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
        addButton.setOnClickListener {
            showAddIngredientDialog()
        }
    }

    private fun loadIngredients() {
        Log.d("MainActivity", "loadIngredients() called")
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
            
            Log.d("MainActivity", "Ingredients loaded and displayed successfully")
        } catch (e: Exception) {
            Log.e("MainActivity", "Failed to load ingredients", e)
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

    private fun updatePantryStatus(ingredient: Ingredient, isInPantry: Boolean) {
        try {
            cookbookEngine.updatePantryStatus(ingredient.name, isInPantry)
            
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

    override fun onDestroy() {
        super.onDestroy()
        if (::cookbookEngine.isInitialized) {
            cookbookEngine.cleanup()
        }
    }
}
