mod vulkan_instance;

fn main() {
    unsafe {vulkan_instance::VulkanInstance::new(1280, 720); }
}
