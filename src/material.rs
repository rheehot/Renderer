use alloc::{sync::Arc, vec::Vec};

use hashbrown::HashMap;

use crate::{buffer::Buffer, Renderer, Shader, ShaderBindingType, Texture};

pub struct Material {
    pub(crate) vertex_shader: Arc<Shader>,
    pub(crate) fragment_shader: Arc<Shader>,
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) bind_group: wgpu::BindGroup,

    _textures: HashMap<&'static str, Arc<Texture>>,
    _uniforms: HashMap<&'static str, Arc<Buffer>>,
}

impl Material {
    pub fn new(
        renderer: &Renderer,
        textures: HashMap<&'static str, Arc<Texture>>,
        uniforms: HashMap<&'static str, Arc<Buffer>>,
        vertex_shader: Arc<Shader>,
        fragment_shader: Arc<Shader>,
    ) -> Self {
        let vs_bindings = vertex_shader.wgpu_bindings(wgpu::ShaderStage::VERTEX);
        let fs_bindings = fragment_shader.wgpu_bindings(wgpu::ShaderStage::FRAGMENT);

        // TODO split bind groups by stage..
        let bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &vs_bindings.into_iter().chain(fs_bindings.into_iter()).collect::<Vec<_>>(),
            label: None,
        });
        let pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        // TODO wip
        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        let bindings = vertex_shader
            .bindings
            .iter()
            .chain(fragment_shader.bindings.iter())
            .map(|(binding_name, binding)| {
                let resource = match binding.binding_type {
                    ShaderBindingType::UniformBuffer => {
                        if *binding_name == "Mvp" {
                            renderer.mvp_buf.binding_resource()
                        } else {
                            let buffer = uniforms.get(binding_name);
                            match buffer {
                                Some(x) => x.binding_resource(),
                                None => panic!("No such buffer named {}", binding_name),
                            }
                        }
                    }
                    ShaderBindingType::Texture2D => {
                        let texture = textures.get(binding_name);
                        match texture {
                            Some(x) => wgpu::BindingResource::TextureView(&x.texture_view),
                            None => panic!("No such texture named {}", binding_name),
                        }
                    }
                    ShaderBindingType::Sampler => wgpu::BindingResource::Sampler(&sampler),
                };

                wgpu::Binding {
                    binding: binding.binding,
                    resource,
                }
            })
            .collect::<Vec<_>>();

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &bindings,
            label: None,
        });

        Self {
            vertex_shader,
            fragment_shader,
            pipeline_layout,
            bind_group,
            _textures: textures,
            _uniforms: uniforms,
        }
    }
}
