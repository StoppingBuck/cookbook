package com.example.pantryman

import android.app.Activity
import android.content.Intent
import android.content.SharedPreferences
import android.net.Uri
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.documentfile.provider.DocumentFile
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.InputStream
import java.io.OutputStream

class SettingsActivity : AppCompatActivity() {
    
    private lateinit var currentDataDirText: TextView
    private lateinit var selectDataDirButton: Button
    private lateinit var sharedPreferences: SharedPreferences
    
    companion object {
        private const val REQUEST_CODE_SELECT_DIRECTORY = 1001
        private const val PREFS_NAME = "PantrymanPrefs"
        private const val PREF_DATA_DIR = "data_directory"
        private const val DEFAULT_DATA_DIR = "cookbook_data" // relative to internal storage
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_settings)
        
        sharedPreferences = getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
        
        initializeViews()
        updateCurrentDataDir()
        setupListeners()
    }
    
    private fun initializeViews() {
        currentDataDirText = findViewById(R.id.textViewCurrentDataDir)
        selectDataDirButton = findViewById(R.id.buttonSelectDataDir)
    }
    
    private fun updateCurrentDataDir() {
        val currentDir = getCurrentDataDir()
        currentDataDirText.text = "Current: $currentDir"
    }
    
    private fun getCurrentDataDir(): String {
        val savedPath = sharedPreferences.getString(PREF_DATA_DIR, null)
        return savedPath ?: "${filesDir.absolutePath}/$DEFAULT_DATA_DIR"
    }
    
    private fun setupListeners() {
        selectDataDirButton.setOnClickListener {
            openDirectoryPicker()
        }
    }
    
    private fun openDirectoryPicker() {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or 
                   Intent.FLAG_GRANT_WRITE_URI_PERMISSION or
                   Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
        }
        
        try {
            startActivityForResult(intent, REQUEST_CODE_SELECT_DIRECTORY)
        } catch (e: Exception) {
            Log.e("SettingsActivity", "Failed to open directory picker", e)
            showError("Failed to open directory picker: ${e.message}")
        }
    }
    
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        
        if (requestCode == REQUEST_CODE_SELECT_DIRECTORY && resultCode == Activity.RESULT_OK) {
            data?.data?.let { uri ->
                handleSelectedDirectory(uri)
            }
        }
    }
    
    private fun handleSelectedDirectory(uri: Uri) {
        try {
            // Grant persistent permission
            contentResolver.takePersistableUriPermission(
                uri, 
                Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            )
            
            val selectedDir = DocumentFile.fromTreeUri(this, uri)
            if (selectedDir == null || !selectedDir.exists()) {
                showError("Selected directory is not accessible")
                return
            }
            
            Log.d("SettingsActivity", "Selected directory: ${selectedDir.name} at $uri")
            
            // Check if directory is empty or contains valid cookbook data
            val isValidCookbookDir = isValidCookbookDirectory(selectedDir)
            val isEmpty = isDirectoryEmpty(selectedDir)
            
            when {
                isEmpty -> {
                    showMigrationDialog(uri, "The selected directory is empty. Move your current data there?")
                }
                isValidCookbookDir -> {
                    showSwitchDialog(uri, "The selected directory contains valid cookbook data. Switch to using this data?")
                }
                else -> {
                    showError("The selected directory is not empty and does not contain valid cookbook data. Please choose an empty directory or one with valid cookbook files.")
                }
            }
            
        } catch (e: Exception) {
            Log.e("SettingsActivity", "Failed to handle selected directory", e)
            showError("Failed to process selected directory: ${e.message}")
        }
    }
    
    private fun isDirectoryEmpty(dir: DocumentFile): Boolean {
        return dir.listFiles().isEmpty()
    }
    
    private fun isValidCookbookDirectory(dir: DocumentFile): Boolean {
        // Check for required structure: ingredients/ folder and pantry.yaml
        val hasIngredients = dir.listFiles().any { 
            it.isDirectory && it.name == "ingredients" 
        }
        val hasPantry = dir.listFiles().any { 
            it.isFile && it.name == "pantry.yaml" 
        }
        
        if (!hasIngredients || !hasPantry) {
            return false
        }
        
        // Check if ingredients folder has at least one YAML file
        val ingredientsDir = dir.listFiles().find { 
            it.isDirectory && it.name == "ingredients" 
        }
        val hasIngredientFiles = ingredientsDir?.listFiles()?.any { 
            it.isFile && it.name?.endsWith(".yaml") == true 
        } ?: false
        
        return hasIngredientFiles
    }
    
    private fun showMigrationDialog(uri: Uri, message: String) {
        AlertDialog.Builder(this)
            .setTitle("Move Data")
            .setMessage(message)
            .setPositiveButton("Yes") { _, _ ->
                migrateDataToNewDirectory(uri)
            }
            .setNegativeButton("Cancel") { dialog, _ ->
                dialog.dismiss()
            }
            .show()
    }
    
    private fun showSwitchDialog(uri: Uri, message: String) {
        AlertDialog.Builder(this)
            .setTitle("Switch Data Directory")
            .setMessage(message)
            .setPositiveButton("Yes") { _, _ ->
                switchToNewDirectory(uri)
            }
            .setNegativeButton("Cancel") { dialog, _ ->
                dialog.dismiss()
            }
            .show()
    }
    
    private fun migrateDataToNewDirectory(uri: Uri) {
        try {
            updateCurrentDataDir() // Update display first
            
            val sourceDir = File(getCurrentSourceDataDirectory())
            val targetDir = DocumentFile.fromTreeUri(this, uri)
            
            if (targetDir == null) {
                showError("Failed to access target directory")
                return
            }
            
            Log.d("SettingsActivity", "Starting migration from ${sourceDir.absolutePath} to ${targetDir.uri}")
            
            // Create cookbook data structure in target directory
            val ingredientsDir = targetDir.createDirectory("ingredients")
            val recipesDir = targetDir.createDirectory("recipes")
            
            if (ingredientsDir == null || recipesDir == null) {
                showError("Failed to create directory structure in target location")
                return
            }
            
            // Copy pantry.yaml
            val pantryFile = File(sourceDir, "pantry.yaml")
            if (pantryFile.exists()) {
                val pantryTarget = targetDir.createFile("application/x-yaml", "pantry.yaml")
                if (pantryTarget != null) {
                    copyFileToDocument(pantryFile, pantryTarget)
                    Log.d("SettingsActivity", "Copied pantry.yaml")
                }
            }
            
            // Copy ingredients directory
            val sourceIngredients = File(sourceDir, "ingredients")
            if (sourceIngredients.exists() && sourceIngredients.isDirectory()) {
                sourceIngredients.listFiles()?.forEach { file ->
                    if (file.isFile && file.name.endsWith(".yaml")) {
                        val targetFile = ingredientsDir.createFile("application/x-yaml", file.name)
                        if (targetFile != null) {
                            copyFileToDocument(file, targetFile)
                            Log.d("SettingsActivity", "Copied ingredient: ${file.name}")
                        }
                    }
                }
            }
            
            // Copy recipes directory
            val sourceRecipes = File(sourceDir, "recipes")
            if (sourceRecipes.exists() && sourceRecipes.isDirectory()) {
                sourceRecipes.listFiles()?.forEach { file ->
                    if (file.isFile && file.name.endsWith(".md")) {
                        val targetFile = recipesDir.createFile("text/markdown", file.name)
                        if (targetFile != null) {
                            copyFileToDocument(file, targetFile)
                            Log.d("SettingsActivity", "Copied recipe: ${file.name}")
                        }
                    }
                }
            }
            
            // Save the new directory preference
            val uriString = uri.toString()
            sharedPreferences.edit()
                .putString(PREF_DATA_DIR, uriString)
                .apply()
                
            updateCurrentDataDir()
            
            // Notify MainActivity of the change and switch without restart
            notifyMainActivityOfDirectoryChange()
            
            Toast.makeText(this, "Data migration completed successfully!", Toast.LENGTH_LONG).show()
            
        } catch (e: Exception) {
            Log.e("SettingsActivity", "Failed to migrate data", e)
            showError("Failed to migrate data: ${e.message}")
        }
    }
    
    private fun switchToNewDirectory(uri: Uri) {
        try {
            val uriString = uri.toString()
            sharedPreferences.edit()
                .putString(PREF_DATA_DIR, uriString)
                .apply()
                
            updateCurrentDataDir()
            
            // Notify MainActivity of the change and switch without restart
            notifyMainActivityOfDirectoryChange()
            
            Toast.makeText(this, "Switched to new data directory successfully!", Toast.LENGTH_LONG).show()
            
        } catch (e: Exception) {
            Log.e("SettingsActivity", "Failed to switch directory", e)
            showError("Failed to switch directory: ${e.message}")
        }
    }
    
    /**
     * Copy a file from internal storage to a DocumentFile location
     */
    private fun copyFileToDocument(sourceFile: File, targetDocument: DocumentFile) {
        try {
            sourceFile.inputStream().use { input ->
                contentResolver.openOutputStream(targetDocument.uri)?.use { output ->
                    input.copyTo(output)
                }
            }
        } catch (e: Exception) {
            Log.e("SettingsActivity", "Failed to copy file ${sourceFile.name}", e)
            throw e
        }
    }
    
    /**
     * Get the current source data directory (for migration purposes)
     */
    private fun getCurrentSourceDataDirectory(): String {
        val savedPath = sharedPreferences.getString(PREF_DATA_DIR, null)
        return if (savedPath?.startsWith("content://") == true) {
            // If we're currently using a content URI, fall back to the default internal storage
            "${filesDir.absolutePath}/$DEFAULT_DATA_DIR"
        } else {
            savedPath ?: "${filesDir.absolutePath}/$DEFAULT_DATA_DIR"
        }
    }
    
    /**
     * Notify MainActivity that the data directory has changed so it can reinitialize
     */
    private fun notifyMainActivityOfDirectoryChange() {
        try {
            // Send a result back to MainActivity indicating the directory changed
            setResult(Activity.RESULT_OK, Intent().apply {
                putExtra("data_directory_changed", true)
            })
        } catch (e: Exception) {
            Log.w("SettingsActivity", "Failed to notify MainActivity of directory change", e)
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
}
