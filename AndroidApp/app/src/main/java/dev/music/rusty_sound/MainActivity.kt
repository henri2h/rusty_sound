package dev.music.rusty_sound

import android.app.NativeActivity

class MainActivity : NativeActivity() {
    override fun onStart() {
        super.onStart()
        if (checkSelfPermission(android.Manifest.permission.RECORD_AUDIO)
            != android.content.pm.PackageManager.PERMISSION_GRANTED
        ) {
            requestPermissions(arrayOf(android.Manifest.permission.RECORD_AUDIO), 1)
        }
    }
}
