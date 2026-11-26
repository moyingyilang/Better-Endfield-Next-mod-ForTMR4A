package dev.betterendfield.next.ui.import

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.betterendfield.next.ImportRepository
import dev.betterendfield.next.ImportState
import dev.betterendfield.next.InstalledModel
import dev.betterendfield.next.ui.design.DesignButton
import dev.betterendfield.next.ui.design.DesignCard
import dev.betterendfield.next.ui.design.DesignRow
import dev.betterendfield.next.ui.design.DesignScreen
import dev.betterendfield.next.ui.design.DesignText
import dev.betterendfield.next.ui.design.DesignTheme
import kotlinx.coroutines.delay

/**
 * Compose import page. Accepts every source the native pipeline accepts
 * (BEM / BEM ZIP / EFMI / EFF / RAR / 7z / tar…), delegates to the shared
 * worker through [ImportRepository] and mirrors its progress.
 */
class ImportActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            DesignTheme {
                ImportScreen(onClose = { finish() })
            }
        }
    }
}

@Composable
private fun ImportScreen(onClose: () -> Unit) {
    val context = LocalContext.current
    var state by remember { mutableStateOf(ImportRepository.state()) }
    var installed by remember { mutableStateOf(emptyList<InstalledModel>()) }
    var selectedName by remember { mutableStateOf<String?>(null) }

    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        selectedName = uri.lastPathSegment?.substringAfterLast('/')
        ImportRepository.start(context, uri)
        state = ImportRepository.state()
    }

    LaunchedEffect(Unit) {
        installed = ImportRepository.installed(context)
        while (true) {
            val current = ImportRepository.state()
            if (current.busy != state.busy || current.status != state.status ||
                current.progressPercent != state.progressPercent
            ) {
                state = current
            }
            if (!current.busy) {
                installed = ImportRepository.installed(context)
            }
            delay(400)
        }
    }

    DesignScreen(title = "导入资源包", subtitle = "BEM / ZIP / EFMI / EFF / RAR / 7z", onClose = onClose) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(top = 20.dp)
                .verticalScroll(rememberScrollState()),
        ) {
            DesignCard {
                DesignText("选择模型包", size = 17.sp, bold = true)
                Spacer(Modifier.height(6.dp))
                DesignText(selectedName ?: "支持 BEM、ZIP、EFMI、EFF、RAR、7z、tar 及其压缩", muted = true)
                Spacer(Modifier.height(14.dp))
                DesignRow {
                    DesignButton("选择文件", onClick = { picker.launch(arrayOf("*/*")) })
                    if (state.busy) {
                        DesignButton("取消", primary = false, onClick = { ImportRepository.cancel() })
                    }
                }
            }

            Spacer(Modifier.height(12.dp))

            DesignCard {
                DesignText("状态", size = 17.sp, bold = true)
                Spacer(Modifier.height(8.dp))
                DesignText(state.status.ifBlank { "空闲" }, muted = !state.busy)
                if (state.busy && state.progressPercent in 0..100) {
                    Spacer(Modifier.height(12.dp))
                    ImportProgress(state.progressPercent)
                }
            }

            Spacer(Modifier.height(12.dp))

            DesignCard {
                DesignText("已安装模型", size = 17.sp, bold = true)
                Spacer(Modifier.height(10.dp))
                if (installed.isEmpty()) {
                    DesignText("还没有模型包", muted = true)
                } else {
                    installed.forEach { model ->
                        DesignRow {
                            Box(
                                modifier = Modifier
                                    .size(6.dp)
                                    .clip(CircleShape)
                                    .background(MaterialTheme.colorScheme.primary),
                            )
                            Column(modifier = Modifier.align(Alignment.CenterVertically)) {
                                DesignText(model.name)
                                if (model.character.isNotEmpty()) {
                                    DesignText(model.character, size = 12.sp, muted = true)
                                }
                            }
                        }
                        Spacer(Modifier.height(8.dp))
                    }
                }
            }

            Spacer(Modifier.height(24.dp))
        }
    }
}

@Composable
private fun ImportProgress(percent: Int) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(6.dp)
            .clip(RoundedCornerShape(3.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth(percent / 100f)
                .fillMaxHeight()
                .background(MaterialTheme.colorScheme.primary),
        )
    }
}
