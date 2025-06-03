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
            // TODO: Implement migration logic
            // For now, just save the new directory and show a message
            val uriString = uri.toString()
            sharedPreferences.edit()
                .putString(PREF_DATA_DIR, uriString)
                .apply()
                
            updateCurrentDataDir()
            Toast.makeText(this, "Data directory updated. Restart the app to use the new location.", Toast.LENGTH_LONG).show()
            
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
            Toast.makeText(this, "Data directory updated. Restart the app to use the new data.", Toast.LENGTH_LONG).show()
            
        } catch (e: Exception) {
            Log.e("SettingsActivity", "Failed to switch directory", e)
            showError("Failed to switch directory: ${e.message}")
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
