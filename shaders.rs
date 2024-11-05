use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, current_shader: u32) -> Color {
    match current_shader {
        0 => sun_shader(fragment, uniforms),            // Shader de Sol estilo lava
        1 => earth_clouds(fragment, uniforms),          // Shader de Tierra con nubes
        2 => noise_shader(fragment, uniforms),          // Shader de ruido para manchas dinámicas
        3 => moon_shader_bright_craters(fragment, uniforms), // Shader de Luna con cráteres brillantes
        4 => ripple_shader(fragment, uniforms),         // Shader de ondas
        5 => dynamic_cellular_shader(fragment, uniforms), // Nuevo shader dinámico celular
        _ => dynamic_cellular_shader(fragment, uniforms),        // Shader por defecto
    }
}

fn ripple_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Posición del fragmento
    let pos = fragment.vertex_position;
    
    // Configuración de la onda
    let wave_speed = 0.3;
    let wave_frequency = 10.0;
    let wave_amplitude = 0.05;
    let time = uniforms.time as f32 * wave_speed;

    // Calcular el desplazamiento basado en el ruido y la onda
    let distance = (pos.x.powi(2) + pos.y.powi(2)).sqrt();
    let ripple = (wave_frequency * (distance - time)).sin() * wave_amplitude;

    // Colores de las ondas
    let base_color = Color::new(70, 130, 180); // Azul acero
    let ripple_color = Color::new(173, 216, 230); // Azul claro

    // Mezclar los colores basados en el valor de la onda
    let color_factor = ripple.clamp(0.0, 1.0);
    let final_color = base_color.lerp(&ripple_color, color_factor);

    // Aplicar intensidad para simular iluminación
    final_color * fragment.intensity
}


/// Shader para namecusein
fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0; // Zoom para el patrón de ruido
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let time = uniforms.time as f32 * 0.01; // Tiempo para animar el patrón

    // Obtener el valor de ruido en 2D con desplazamiento temporal para movimiento
    let noise_value = uniforms.noise.get_noise_2d(x * zoom + time, y * zoom + time);

    // Definir los colores de las manchas solares
    let bright_color = Color::new(255, 255, 102); // Amarillo brillante para áreas calientes
    let dark_spot_color = Color::new(139, 0, 0);  // Rojo oscuro para manchas solares
    let base_color = Color::new(255, 69, 0);      // Rojo anaranjado para la superficie

    // Umbral para decidir entre zonas brillantes y oscuras
    let spot_threshold = 0.6;

    // Determinar el color basado en el valor de ruido
    let noise_color = if noise_value < spot_threshold {
        bright_color // Zonas brillantes
    } else {
        dark_spot_color // Manchas solares
    };

    // Mezclar el color base con el color determinado por el ruido
    let final_color = base_color.lerp(&noise_color, noise_value.clamp(0.0, 1.0));

    // Ajustar la intensidad para simular iluminación
    final_color * fragment.intensity
}

fn noise_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Posición del fragmento
    let pos = fragment.vertex_position;

    // Radio del círculo
    let radius = 0.1;

    // Velocidad del movimiento
    let speed = 0.2;
    let time = uniforms.time as f32 * 0.01;  // Escala de tiempo para el movimiento

    // Luz direccional
    let light_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
    let normal = fragment.normal.normalize();
    let intensity = normal.dot(&light_dir).max(0.0);

    // Círculos en movimiento
    let mut circle_mask = 0.0;
    for i in -3..=3 {
        for j in -3..=3 {
            // Desplazamiento dinámico basado en el tiempo
            let offset_x = (i as f32 * 0.3) + (time * speed);
            let offset_y = (j as f32 * 0.3) + (time * speed * 0.5);

            let dist_to_circle = ((pos.x - offset_x).powi(2) + (pos.y - offset_y).powi(2)).sqrt();

            if dist_to_circle < radius {
                circle_mask = 1.0;
                break;
            }
        }
        if circle_mask == 1.0 {
            break;
        }
    }

    // Determinar el color basado en si está dentro del círculo
    if circle_mask > 0.5 {
        // Círculo negro con sombreado basado en la luz
        Color::new(0, 0, 0) * intensity
    } else {
        // Fondo blanco también afectado por la luz para un toque más realista
        Color::new(255, 255, 255) * (0.5 + intensity * 0.5)
    }
}


fn moon_shader_bright_craters(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.1;

    // Añadimos un efecto pulsante a los cráteres
    let pulsate = (t * 0.5).sin() * 0.05;

    // Ruido para la textura de la superficie
    let surface_noise = uniforms.noise.get_noise_2d(x * zoom + t, y * zoom + t);

    let gray_color = Color::new(200, 200, 200);
    let bright_crater_color = Color::new(220, 220, 220); // Cráter más brillante
    let dynamic_color = Color::new(250, 250, 250); // Toque dinámico brillante

    let crater_threshold = 0.4 + pulsate; // Dinamismo en los cráteres

    // Definir el color base de la luna
    let base_color = if surface_noise > crater_threshold {
        gray_color
    } else if surface_noise > crater_threshold - 0.1 {
        bright_crater_color
    } else {
        dynamic_color // Zonas más dinámicas
    };

    base_color * fragment.intensity
}

fn earth_clouds(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 80.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.1;

    // Ruido para la superficie terrestre
    let surface_noise = uniforms.noise.get_noise_2d(x * zoom + t, y * zoom);

    let ocean_color = Color::new(0, 105, 148);     // Azul océano
    let land_color = Color::new(34, 139, 34);      // Verde tierra
    let desert_color = Color::new(210, 180, 140);  // Marrón desierto
    let snow_color = Color::new(255, 250, 250);    // Blanco nieve

    // Umbrales para definir las diferentes zonas geográficas
    let snow_threshold = 0.7;
    let land_threshold = 0.4;
    let desert_threshold = 0.3;

    // Selección de color base
    let base_color = if y.abs() > snow_threshold {
        snow_color
    } else if surface_noise > land_threshold {
        land_color
    } else if surface_noise > desert_threshold {
        desert_color
    } else {
        ocean_color
    };

    // Dinámica de nubes
    let cloud_zoom = 100.0; // Ajuste para las nubes
    let cloud_noise = uniforms.noise.get_noise_2d(x * cloud_zoom + t * 0.5, y * cloud_zoom + t * 0.5);

    // Crear nubes dinámicas y movimiento
    let cloud_color = Color::new(255, 255, 255); // Blanco para nubes
    let sky_gradient = Color::new(135, 206, 250); // Azul cielo claro

    let cloud_intensity = cloud_noise.clamp(0.4, 0.7) - 0.4;
    let final_color = if cloud_noise > 0.6 {
        base_color.lerp(&cloud_color, cloud_intensity * 0.5) // Mezcla el color base con nubes
    } else {
        base_color.lerp(&sky_gradient, 0.1) // Mezcla con el gradiente del cielo
    };

    final_color * fragment.intensity
}

fn dynamic_cellular_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 30.0;  // Escala del patrón celular
    let flow_speed = 0.1; // Velocidad del flujo
    let time = uniforms.time as f32 * flow_speed; // Tiempo para animación

    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Usar `get_noise_2d` con tiempo para animación controlada
    let cell_noise_value = uniforms.noise.get_noise_2d(x * zoom, y * zoom + time).abs();

    // Definir colores dinámicos para las células
    let energy_color_1 = Color::new(255, 69, 0);    // Naranja brillante
    let energy_color_2 = Color::new(255, 140, 0);   // Naranja más suave
    let energy_color_3 = Color::new(255, 215, 0);   // Amarillo dorado
    let energy_color_4 = Color::new(255, 255, 153); // Amarillo pálido

    // Selección de color basado en el valor de ruido
    let final_color = if cell_noise_value < 0.2 {
        energy_color_1
    } else if cell_noise_value < 0.5 {
        energy_color_2
    } else if cell_noise_value < 0.8 {
        energy_color_3
    } else {
        energy_color_4
    };

    // Ajustar intensidad para simular efectos de iluminación
    final_color * fragment.intensity
}


fn default_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    fragment.color // Devuelve el color original del fragmento
}