val imguiJavaVersion: String by project
val lwjglVersion: String by project
val gsonVersion: String by project
val javaWebSocketVersion: String by project
val junitVersion: String by project
val hostVersion: String by project

plugins {
    application
}

application {
    mainClass.set("com.lattestudio.imguistorybook.host.HostMain")
    applicationDefaultJvmArgs = listOf("-Dorg.lwjgl.util.Debug=false")
}

dependencies {
    api(project(":api"))

    implementation(platform("org.lwjgl:lwjgl-bom:$lwjglVersion"))
    implementation("org.lwjgl:lwjgl")
    implementation("org.lwjgl:lwjgl-glfw")
    implementation("org.lwjgl:lwjgl-opengl")
    implementation("org.lwjgl:lwjgl-stb")
    runtimeOnly("org.lwjgl:lwjgl::natives-windows")
    runtimeOnly("org.lwjgl:lwjgl-glfw::natives-windows")
    runtimeOnly("org.lwjgl:lwjgl-opengl::natives-windows")
    runtimeOnly("org.lwjgl:lwjgl-stb::natives-windows")

    implementation("io.github.spair:imgui-java-lwjgl3:$imguiJavaVersion")
    runtimeOnly("io.github.spair:imgui-java-natives-windows:$imguiJavaVersion")

    implementation("com.google.code.gson:gson:$gsonVersion")
    implementation("org.java-websocket:Java-WebSocket:$javaWebSocketVersion")

    testImplementation(platform("org.junit:junit-bom:$junitVersion"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testImplementation(project(":api"))
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.processResources {
    filesMatching("host.properties") {
        expand(
            "hostVersion" to hostVersion,
            "imguiJavaVersion" to imguiJavaVersion,
        )
    }
}
