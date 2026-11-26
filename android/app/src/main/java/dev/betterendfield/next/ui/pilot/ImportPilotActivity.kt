package dev.betterendfield.next.ui.pilot

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.betterendfield.next.ui.design.DesignButton
import dev.betterendfield.next.ui.design.DesignCard
import dev.betterendfield.next.ui.design.DesignRow
import dev.betterendfield.next.ui.design.DesignScreen
import dev.betterendfield.next.ui.design.DesignSystem
import dev.betterendfield.next.ui.design.DesignText
import dev.betterendfield.next.ui.design.DesignTheme
import dev.betterendfield.next.ui.import.ImportActivity
import java.io.File

/**
 * Compose pilot for the model page. Coexists with the upstream Java/XML pages;
 * each page is replaced by a Compose screen once it reaches parity. The screen
 * is written against the semantic design components, so switching the design
 * system re-renders the same page with Endfield / Material / MIUIX styling.
 */
class ImportPilotActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            val context = LocalContext.current
            var system by remember { mutableStateOf(DesignSystem.read(context)) }
            DesignTheme(system) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.background),
                ) {
                    ModelHomeScreen(
                        system = system,
                        onSelectSystem = { selected ->
                            DesignSystem.write(context, selected)
                            system = selected
                        },
                        onClose = { finish() },
                        onImport = {
                            startActivity(Intent(context, ImportActivity::class.java))
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun ModelHomeScreen(
    system: DesignSystem,
    onSelectSystem: (DesignSystem) -> Unit,
    onClose: () -> Unit,
    onImport: () -> Unit,
) {
    val context = androidx.compose.ui.platform.LocalContext.current
    var generationCount by remember { mutableIntStateOf(countGenerations(context.filesDir)) }

    DesignScreen(title = "模型", subtitle = "Better Endfield Next", onClose = onClose) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(top = 20.dp)
                .verticalScroll(rememberScrollState()),
        ) {
            DesignCard {
                DesignText("资源与导入", size = 17.sp, bold = true)
                Spacer(Modifier.height(6.dp))
                DesignText("已安装 $generationCount 个模型包", muted = true)
                Spacer(Modifier.height(14.dp))
                DesignRow {
                    DesignButton("导入资源包", onClick = onImport)
                    DesignButton("刷新", primary = false) {
                        generationCount = countGenerations(context.filesDir)
                    }
                }
            }

            Spacer(Modifier.height(12.dp))

            DesignCard {
                DesignText("设计系统", size = 17.sp, bold = true)
                Spacer(Modifier.height(10.dp))
                DesignRow {
                    DesignSystem.entries.forEach { entry ->
                        DesignButton(entry.label, primary = entry == system) {
                            onSelectSystem(entry)
                        }
                    }
                }
            }

            Spacer(Modifier.height(12.dp))

            DesignCard {
                DesignText("已安装模型", size = 17.sp, bold = true)
                Spacer(Modifier.height(10.dp))
                val generations = generations(context.filesDir)
                if (generations.isEmpty()) {
                    DesignText("还没有模型包", muted = true)
                } else {
                    generations.forEach { name ->
                        DesignRow {
                            Box(
                                modifier = Modifier
                                    .size(6.dp)
                                    .clip(CircleShape)
                                    .background(MaterialTheme.colorScheme.primary),
                            )
                            DesignText(name)
                        }
                        Spacer(Modifier.height(8.dp))
                    }
                }
            }

            Spacer(Modifier.height(24.dp))
        }
    }
}

private fun generations(filesDir: File): List<String> =
    File(filesDir, "bem-installed").listFiles()
        ?.filter { it.isDirectory && !it.name.startsWith("stage-") }
        ?.map { it.name }
        ?.sorted()
        ?: emptyList()

private fun countGenerations(filesDir: File): Int = generations(filesDir).size
