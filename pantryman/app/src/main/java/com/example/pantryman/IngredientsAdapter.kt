package com.example.pantryman

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.CheckBox
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView

/**
 * RecyclerView adapter for displaying ingredients grouped by category
 */
class IngredientsAdapter(
    private val onIngredientClick: (Ingredient) -> Unit,
    private val onPantryStatusChange: (Ingredient, Boolean) -> Unit
) : RecyclerView.Adapter<RecyclerView.ViewHolder>() {
    
    private var items = listOf<DisplayItem>()
    
    companion object {
        private const val TYPE_CATEGORY_HEADER = 0
        private const val TYPE_INGREDIENT = 1
    }
    
    sealed class DisplayItem {
        data class CategoryHeader(val category: String) : DisplayItem()
        data class IngredientItem(val ingredient: Ingredient) : DisplayItem()
    }
    
    fun updateData(ingredientsByCategory: Map<String, List<Ingredient>>) {
        val newItems = mutableListOf<DisplayItem>()
        
        // Sort categories alphabetically
        val sortedCategories = ingredientsByCategory.keys.sorted()
        
        for (category in sortedCategories) {
            newItems.add(DisplayItem.CategoryHeader(category))
            
            // Sort ingredients within category alphabetically
            val sortedIngredients = ingredientsByCategory[category]?.sortedBy { it.name } ?: emptyList()
            for (ingredient in sortedIngredients) {
                newItems.add(DisplayItem.IngredientItem(ingredient))
            }
        }
        
        items = newItems
        notifyDataSetChanged()
    }
    
    fun updateIngredients(ingredients: List<Ingredient>) {
        // Group ingredients by category
        val ingredientsByCategory = ingredients.groupBy { it.category }
        updateData(ingredientsByCategory)
    }
    
    override fun getItemViewType(position: Int): Int {
        return when (items[position]) {
            is DisplayItem.CategoryHeader -> TYPE_CATEGORY_HEADER
            is DisplayItem.IngredientItem -> TYPE_INGREDIENT
        }
    }
    
    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RecyclerView.ViewHolder {
        return when (viewType) {
            TYPE_CATEGORY_HEADER -> {
                val view = LayoutInflater.from(parent.context)
                    .inflate(R.layout.item_category_header, parent, false)
                CategoryHeaderViewHolder(view)
            }
            TYPE_INGREDIENT -> {
                val view = LayoutInflater.from(parent.context)
                    .inflate(R.layout.item_ingredient, parent, false)
                IngredientViewHolder(view)
            }
            else -> throw IllegalArgumentException("Unknown view type: $viewType")
        }
    }
    
    override fun onBindViewHolder(holder: RecyclerView.ViewHolder, position: Int) {
        when (val item = items[position]) {
            is DisplayItem.CategoryHeader -> {
                (holder as CategoryHeaderViewHolder).bind(item.category)
            }
            is DisplayItem.IngredientItem -> {
                (holder as IngredientViewHolder).bind(item.ingredient)
            }
        }
    }
    
    override fun getItemCount(): Int = items.size
    
    inner class CategoryHeaderViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        private val categoryText: TextView = itemView.findViewById(R.id.categoryText)
        
        fun bind(category: String) {
            categoryText.text = category
        }
    }
    
    inner class IngredientViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        private val ingredientName: TextView = itemView.findViewById(R.id.ingredientName)
        private val ingredientDetails: TextView = itemView.findViewById(R.id.ingredientDetails)
        private val pantryCheckbox: CheckBox = itemView.findViewById(R.id.pantryCheckbox)
        
        fun bind(ingredient: Ingredient) {
            ingredientName.text = ingredient.name
            
            // Show quantity and type if in stock
            val details = if (ingredient.isInPantry) {
                val quantity = ingredient.quantity?.toString() ?: "?"
                val unit = ingredient.quantityType?.takeIf { it.isNotEmpty() } ?: ""
                "In Stock: $quantity $unit".trim()
            } else {
                "Not in stock"
            }
            ingredientDetails.text = details
            
            // Set checkbox state without triggering listener
            pantryCheckbox.setOnCheckedChangeListener(null)
            pantryCheckbox.isChecked = ingredient.isInPantry
            
            // Set up listeners
            pantryCheckbox.setOnCheckedChangeListener { _, isChecked ->
                onPantryStatusChange(ingredient, isChecked)
            }
            
            itemView.setOnClickListener {
                onIngredientClick(ingredient)
            }
        }
    }
}
