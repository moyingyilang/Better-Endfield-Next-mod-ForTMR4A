package dev.betterendfield.next;

import android.content.Context;
import android.content.SharedPreferences;

/** Package-private settings bridge for the Kotlin UI layer. */
public final class SettingsStore {
    private SettingsStore() {}

    public static SharedPreferences open(Context context) {
        return FrameworkSettings.open(context);
    }
}
