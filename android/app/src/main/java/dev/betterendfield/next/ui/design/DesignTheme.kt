package dev.betterendfield.next.ui.design

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.platform.LocalContext
import dev.betterendfield.next.ui.theme.EndfieldTheme
import top.yukonga.miuix.kmp.theme.ColorSchemeMode
import top.yukonga.miuix.kmp.theme.MiuixTheme
import top.yukonga.miuix.kmp.theme.ThemeController

val LocalDesignSystem = staticCompositionLocalOf { DesignSystem.ENDFIELD }

/**
 * Wraps content with the selected design system's theme. Material Design and
 * Endfield share the Material3 runtime; MIUIX stacks MiuixTheme on top (the
 * Material theme remains available for platform surfaces and fallbacks).
 */
@Composable
fun DesignTheme(
    system: DesignSystem = DesignSystem.read(LocalContext.current),
    content: @Composable () -> Unit,
) {
    CompositionLocalProvider(LocalDesignSystem provides system) {
        when (system) {
            DesignSystem.ENDFIELD -> EndfieldTheme { content() }
            DesignSystem.MATERIAL -> MaterialDesignTheme { content() }
            DesignSystem.MIUIX -> {
                val controller = remember { ThemeController(ColorSchemeMode.System) }
                MiuixTheme(controller = controller) {
                    MaterialDesignTheme { content() }
                }
            }
        }
    }
}

@Composable
private fun MaterialDesignTheme(content: @Composable () -> Unit) {
    val context = LocalContext.current
    val dark = isSystemInDarkTheme()
    val scheme = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        if (dark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
    } else if (dark) {
        darkColorScheme()
    } else {
        lightColorScheme()
    }
    MaterialTheme(colorScheme = scheme, content = content)
}
