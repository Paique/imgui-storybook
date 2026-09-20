val imguiJavaVersion: String by project
val junitVersion: String by project

dependencies {
    api("io.github.spair:imgui-java-binding:$imguiJavaVersion")

    testImplementation(platform("org.junit:junit-bom:$junitVersion"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}
