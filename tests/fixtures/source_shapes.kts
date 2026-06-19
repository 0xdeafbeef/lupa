plugins {
    id("com.android.application")
}

val endpoint = providers.gradleProperty("endpoint").orElse("")

android {
    namespace = "demo"
}

dependencies {
    implementation("x:y:z")
}
