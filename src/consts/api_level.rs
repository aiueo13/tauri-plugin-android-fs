//! Constants representing Android API levels.  
//! 
//! See [`AndroidFs::api_level`](crate::AndroidFs::api_level) for the API level of the current target.
//!
//! # Note
//! Tauri does not support Android versions lower than 7 (API level 24).
//! 
//! # References
//! <https://developer.android.com/guide/topics/manifest/uses-sdk-element#api-level-table>

/// API level for [Build.VERSION_CODES.N](https://developer.android.com/reference/android/os/Build.VERSION_CODES#N)
pub const CODE_N: i32 = 24;

/// API level for [Build.VERSION_CODES.N_MR1](https://developer.android.com/reference/android/os/Build.VERSION_CODES#N_MR1)
pub const CODE_N_MR1: i32 = 25;

/// API level for [Build.VERSION_CODES.O](https://developer.android.com/reference/android/os/Build.VERSION_CODES#O)
pub const CODE_O: i32 = 26;

/// API level for [Build.VERSION_CODES.O_MR1](https://developer.android.com/reference/android/os/Build.VERSION_CODES#O_MR1)
pub const CODE_O_MR1: i32 = 27;

/// API level for [Build.VERSION_CODES.P](https://developer.android.com/reference/android/os/Build.VERSION_CODES#P)
pub const CODE_P: i32 = 28;

/// API level for [Build.VERSION_CODES.Q](https://developer.android.com/reference/android/os/Build.VERSION_CODES#Q)
pub const CODE_Q: i32 = 29;

/// API level for [Build.VERSION_CODES.R](https://developer.android.com/reference/android/os/Build.VERSION_CODES#R)
pub const CODE_R: i32 = 30;

/// API level for [Build.VERSION_CODES.S](https://developer.android.com/reference/android/os/Build.VERSION_CODES#S)
pub const CODE_S: i32 = 31;

/// API level for [Build.VERSION_CODES.S_V2](https://developer.android.com/reference/android/os/Build.VERSION_CODES#S_V2)
pub const CODE_S_V2: i32 = 32;

/// API level for [Build.VERSION_CODES.TIRAMISU](https://developer.android.com/reference/android/os/Build.VERSION_CODES#TIRAMISU)
pub const CODE_TIRAMISU: i32 = 33;

/// API level for [Build.VERSION_CODES.UPSIDE_DOWN_CAKE](https://developer.android.com/reference/android/os/Build.VERSION_CODES#UPSIDE_DOWN_CAKE)
pub const CODE_UPSIDE_DOWN_CAKE: i32 = 34;

/// API level for [Build.VERSION_CODES.VANILLA_ICE_CREAM](https://developer.android.com/reference/android/os/Build.VERSION_CODES#VANILLA_ICE_CREAM)
pub const CODE_VANILLA_ICE_CREAM: i32 = 35;

/// API level for [Build.VERSION_CODES.BAKLAVA](https://developer.android.com/reference/android/os/Build.VERSION_CODES#BAKLAVA)
pub const CODE_BAKLAVA: i32 = 36;

/// API level for Android 7.0
pub const ANDROID_7: i32 = CODE_N;

/// API level for Android 7.1
pub const ANDROID_7_1: i32 = CODE_N_MR1;

/// API level for Android 7.1.1
pub const ANDROID_7_1_1: i32 = CODE_N_MR1;

/// API level for Android 8.0
pub const ANDROID_8: i32 = CODE_O;

/// API level for Android 8.1
pub const ANDROID_8_1: i32 = CODE_O_MR1;

/// API level for Android 9.0
pub const ANDROID_9: i32 = CODE_P;

/// API level for Android 10
pub const ANDROID_10: i32 = CODE_Q;

/// API level for Android 11
pub const ANDROID_11: i32 = CODE_R;

/// API level for Android 12
pub const ANDROID_12: i32 = CODE_S;

/// API level for Android 12L
pub const ANDROID_12_L: i32 = CODE_S_V2;

/// API level for Android 13
pub const ANDROID_13: i32 = CODE_TIRAMISU;

/// API level for Android 14
pub const ANDROID_14: i32 = CODE_UPSIDE_DOWN_CAKE;

/// API level for Android 15
pub const ANDROID_15: i32 = CODE_VANILLA_ICE_CREAM;

/// API level for Android 16
pub const ANDROID_16: i32 = CODE_BAKLAVA;