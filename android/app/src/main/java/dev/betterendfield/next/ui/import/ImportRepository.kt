package dev.betterendfield.next

import android.content.Context
import android.net.Uri
import org.json.JSONArray

/** One installed BEM package as the game sees it. */
data class InstalledModel(
    val generation: String,
    val name: String,
    val character: String,
)

/** Snapshot of the shared import worker. */
data class ImportState(
    val busy: Boolean,
    val status: String,
    val progressPercent: Int,
)

/**
 * Kotlin facade over the upstream import worker. UI code depends on this
 * instead of the Java activity/page classes, which are replaced page by page.
 */
object ImportRepository {
    fun state(): ImportState = ImportState(
        busy = BemInstaller.busy,
        status = BemInstaller.status ?: "",
        progressPercent = BemInstaller.progressPercent,
    )

    fun start(context: Context, uri: Uri): Boolean = BemInstaller.start(context, uri)

    fun cancel() = BemInstaller.cancel()

    fun installed(context: Context): List<InstalledModel> {
        val entries: JSONArray = try {
            BemInstaller.index(context)
        } catch (_: Exception) {
            return emptyList()
        }
        val models = ArrayList<InstalledModel>(entries.length())
        for (i in 0 until entries.length()) {
            val entry = entries.optJSONObject(i) ?: continue
            val generation = entry.optString("generation")
            if (generation.isEmpty()) continue
            models += InstalledModel(
                generation = generation,
                name = entry.optString("name", generation),
                character = entry.optString("target_id", entry.optString("character_id", "")),
            )
        }
        return models
    }
}
