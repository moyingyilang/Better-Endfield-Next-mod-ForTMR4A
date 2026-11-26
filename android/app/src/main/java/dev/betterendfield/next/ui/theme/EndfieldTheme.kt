package dev.betterendfield.next.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

// Endfield UI X tokens (ui-kit/tokens/endfield-ui-x.tokens.json, v0.1.0).
private val DarkScheme = darkColorScheme(
    primary = Color(0xFFFFFA00),
    onPrimary = Color(0xFF191919),
    primaryContainer = Color(0xFF282828),
    onPrimaryContainer = Color(0xFFFFFA00),
    secondary = Color(0xFFB3B3B3),
    onSecondary = Color(0xFF191919),
    background = Color(0xFF141414),
    onBackground = Color(0xFFFAFAFA),
    surface = Color(0xFF1F1F1F),
    onSurface = Color(0xFFFAFAFA),
    surfaceVariant = Color(0xFF282828),
    onSurfaceVariant = Color(0xFFB3B3B3),
    outline = Color(0xFF383838),
    outlineVariant = Color(0xFF383838),
    error = Color(0xFFF62A2A),
    onError = Color(0xFFFAFAFA),
)

private val LightScheme = lightColorScheme(
    primary = Color(0xFFFFFA00),
    onPrimary = Color(0xFF191919),
    primaryContainer = Color(0xFFE6E6E6),
    onPrimaryContainer = Color(0xFF6B5D00),
    secondary = Color(0xFF626262),
    onSecondary = Color(0xFFFFFFFF),
    background = Color(0xFFF2F2F2),
    onBackground = Color(0xFF191919),
    surface = Color(0xFFFFFFFF),
    onSurface = Color(0xFF191919),
    surfaceVariant = Color(0xFFE6E6E6),
    onSurfaceVariant = Color(0xFF626262),
    outline = Color(0xFFD9D9D9),
    outlineVariant = Color(0xFFD9D9D9),
    error = Color(0xFFF62A2A),
    onError = Color(0xFFFFFFFF),
)

private val EndfieldShapes = Shapes(
    extraSmall = RoundedCornerShape(4.dp),
    small = RoundedCornerShape(6.dp),
    medium = RoundedCornerShape(10.dp),
    large = RoundedCornerShape(14.dp),
    extraLarge = RoundedCornerShape(18.dp),
)

@Composable
fun EndfieldTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) DarkScheme else LightScheme,
        shapes = EndfieldShapes,
        typography = Typography(),
        content = content,
    )
}
