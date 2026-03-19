# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# 保持 Tauri 相关类
-keep class com.tauri.** { *; }
-keep class org.json.** { *; }

# 保持 Rust JNI 绑定
-keepclasseswithmembernames class * {
    native <methods>;
}

# 移除日志
-assumenosideeffects class android.util.Log {
    public static *** d(...);
    public static *** v(...);
}

# 压缩 DEX
-dontpreverify

# 保留行号信息用于调试
-keepattributes SourceFile,LineNumberTable

# 如果需要保留源文件名，取消注释以下行
#-renamesourcefileattribute SourceFile