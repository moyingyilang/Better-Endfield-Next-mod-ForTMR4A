package dev.betterendfield.next.ui.design

import android.content.Context
import dev.betterendfield.next.SettingsStore

/**
 * The three supported UI design systems. The choice is a user setting; every
 * Compose screen consumes the abstract components in this package.
 */
enum class DesignSystem(val id: String, val label: String) {
    ENDFIELD("endfield", "Endfield UI X"),
    MATERIAL("material", "Material Design"),
    MIUIX("miuix", "MIUIX");

    companion object {
        const val KEY = "ui.design_system"

        fun read(context: Context): DesignSystem {
            val stored = SettingsStore.open(context).getString(KEY, ENDFIELD.id)
            return entries.firstOrNull { it.id == stored } ?: ENDFIELD
        }

        fun write(context: Context, system: DesignSystem) {
            SettingsStore.open(context).edit().putString(KEY, system.id).apply()
        }
    }
}
