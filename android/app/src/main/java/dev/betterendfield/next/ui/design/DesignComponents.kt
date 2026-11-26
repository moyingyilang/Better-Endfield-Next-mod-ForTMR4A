package dev.betterendfield.next.ui.design

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button as MaterialButton
import androidx.compose.material3.ButtonDefaults as MaterialButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface as MaterialSurface
import androidx.compose.material3.Text as MaterialText
import androidx.compose.material3.TextButton as MaterialTextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import top.yukonga.miuix.kmp.basic.Button as MiuixButton
import top.yukonga.miuix.kmp.basic.ButtonDefaults as MiuixButtonDefaults
import top.yukonga.miuix.kmp.basic.Card as MiuixCard
import top.yukonga.miuix.kmp.basic.Text as MiuixText

/**
 * Semantic components shared by every screen. Each is rendered by the active
 * design system so a page can be written once and appear native in all three.
 */
@Composable
fun DesignText(
    text: String,
    modifier: Modifier = Modifier,
    size: TextUnit = 14.sp,
    bold: Boolean = false,
    muted: Boolean = false,
) {
    if (LocalDesignSystem.current == DesignSystem.MIUIX) {
        MiuixText(
            text = text,
            modifier = modifier,
            fontSize = size,
            fontWeight = if (bold) FontWeight.SemiBold else null,
        )
    } else {
        MaterialText(
            text = text,
            modifier = modifier,
            fontSize = size,
            fontWeight = if (bold) FontWeight.SemiBold else null,
            color = if (muted) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
fun DesignCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    if (LocalDesignSystem.current == DesignSystem.MIUIX) {
        MiuixCard(modifier = modifier.fillMaxWidth()) { content() }
    } else {
        MaterialSurface(
            modifier = modifier.fillMaxWidth(),
            shape = RoundedCornerShape(12.dp),
            color = MaterialTheme.colorScheme.surface,
            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
        ) {
            Column(modifier = Modifier.padding(16.dp)) { content() }
        }
    }
}

@Composable
fun DesignButton(
    text: String,
    modifier: Modifier = Modifier,
    primary: Boolean = true,
    onClick: () -> Unit,
) {
    if (LocalDesignSystem.current == DesignSystem.MIUIX) {
        if (primary) {
            MiuixButton(
                onClick = onClick,
                modifier = modifier,
                colors = MiuixButtonDefaults.buttonColorsPrimary(),
            ) {
                MiuixText(text)
            }
        } else {
            MiuixButton(onClick = onClick, modifier = modifier) {
                MiuixText(text)
            }
        }
    } else if (primary) {
        MaterialButton(
            onClick = onClick,
            modifier = modifier,
            colors = MaterialButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ),
            shape = RoundedCornerShape(8.dp),
        ) {
            MaterialText(text, fontWeight = FontWeight.SemiBold, fontSize = 15.sp)
        }
    } else {
        MaterialTextButton(onClick = onClick, modifier = modifier) {
            MaterialText(text, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
fun DesignScreen(
    title: String,
    subtitle: String? = null,
    onClose: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .systemBarsPadding()
            .padding(horizontal = 20.dp, vertical = 16.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(modifier = Modifier.padding(end = 12.dp)) {
                DesignText(title, size = 30.sp, bold = true)
                if (subtitle != null) {
                    DesignText(subtitle, size = 13.sp, muted = true)
                }
            }
            if (onClose != null) {
                DesignButton("关闭", primary = false, onClick = onClose)
            }
        }
        content()
    }
}

@Composable
fun DesignRow(content: @Composable RowScope.() -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
        content = content,
    )
}

/** Filler kept so screens can reference the theme's accent uniformly. */
@Composable
fun designAccent(): Color = MaterialTheme.colorScheme.primary
