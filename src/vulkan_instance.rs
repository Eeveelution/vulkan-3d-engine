use std::{ffi::{CStr, c_char}, borrow::Cow};

use ash::{Entry, extensions, vk::{KhrPortabilityEnumerationFn, KhrGetPhysicalDeviceProperties2Fn, self, API_VERSION_1_2}};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::LogicalSize};

pub struct VulkanInstance {

}

impl VulkanInstance {
    pub unsafe fn new(window_width: u32, window_height: u32)  {
        let event_loop = EventLoop::new();

        let window = 
            WindowBuilder::new()
                .with_title("Rust & Vulkan 3D Engine")
                .with_inner_size(LogicalSize::new(window_width, window_height))
                .build(&event_loop)
                .expect("Failed to create window!");

        let vk_entry = Entry::load().expect("Failed to load Vulkan!");   

        let app_name = CStr::from_bytes_with_nul_unchecked(b"vulkan-3d-engine-rust-37449\0");

        let layer_names: Vec<*const c_char> = [
            CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")
        ]   .iter()
            .map(|name| { 
                name.as_ptr() 
            }).collect();

        let mut extension_names = 
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .expect("Failed to enumerate over required extensions!")
                .to_vec();

        extension_names.push(extensions::ext::DebugUtils::name().as_ptr());

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
            extension_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
        }

        let app_info = vk::ApplicationInfo::builder()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(API_VERSION_1_2);

        let creation_flags = 
            if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

        let create_info = 
            vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_layer_names(&layer_names)
                .enabled_extension_names(&extension_names)
                .flags(creation_flags);

        let instance = 
            vk_entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan Instance!");

        let debug_information = 
            vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                )
                .pfn_user_callback(
                    Some(vulkan_debug_callback)
                );

        let debug_utils_loader = extensions::ext::DebugUtils::new(&vk_entry, &instance);

        let debug_callback = 
            debug_utils_loader
                .create_debug_utils_messenger(&debug_information, None)
                .expect("Debug Callback could not be created.");

        let surface = 
            ash_window::create_surface(
                &vk_entry, 
                &instance, 
                window.raw_display_handle(), 
                window.raw_window_handle(), 
                None
            )
            .expect("Window Surface creation was unsuccessfull.");

        let physical_devices = 
            instance
                .enumerate_physical_devices()
                .expect("Could not retrieve physical devices!");

        let surface_loader = extensions::khr::Surface::new(&vk_entry, &instance);

        let (physical_device, queue_family_index) = physical_devices
            .iter().find_map(| device | {
                instance
                    .get_physical_device_queue_family_properties(*device)
                    .iter()
                    .enumerate()
                    .find_map(| (index, info) | {
                        let supports_graphics_and_surface = 
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS) 
                        &&
                            surface_loader
                                .get_physical_device_surface_support(*device, index as u32, surface) 
                                .expect("Fetching of physical device surface support failed!");

                        if supports_graphics_and_surface {
                            Some( (*device, index) )
                        } else {
                            None
                        }
                    })  
            }).expect("Suitable Vulkan Device has not been found.");

        let device_extension_names_raw = [
                extensions::khr::Swapchain::name().as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                KhrPortabilitySubsetFn::name().as_ptr(),
            ];

        let priorities = [1.0];

        let queue_info =
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index as u32)
                .queue_priorities(&priorities);

        let device_create_info =
            vk::DeviceCreateInfo::builder()
                .queue_create_infos(
                    std::slice::from_ref(&queue_info)
                )
                .enabled_extension_names(&device_extension_names_raw);

        let vulkan_device =
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Creation of Vulkan device failed!");

        

    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity:       vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type:           vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data:      *mut   std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}