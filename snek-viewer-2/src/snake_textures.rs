use three_d::*;

pub struct SnakeTextures {
    head_up: Texture2DRef,
    head_right: Texture2DRef,
    head_down: Texture2DRef,
    head_left: Texture2DRef,

    // tail_up: Texture2DRef,
    // tail_right: Texture2DRef,
    // tail_down: Texture2DRef,
    // tail_left: Texture2DRef,

    horizontal: Texture2DRef,
    vertical: Texture2DRef,

}

impl SnakeTextures {
    pub fn new(context: &Context) -> Self {
        let mut loaded = three_d_asset::io::load(&[
            "assets/snake-graphics.png",
            "assets/uvchecker.png",
        ])
            .unwrap();

        let cpu_texture: CpuTexture = loaded.deserialize("snake-graphics").unwrap();
        let mut texture = Texture2DRef::from_cpu_texture(
            context,
            &cpu_texture.to_linear_srgb().unwrap(),
        );

        let mut head_up = texture.clone();
        head_up.transformation =
            Matrix3::from_translation(vec2(0.6, 0.8)) *
                Matrix3::from_scale(0.2);

        let mut head_right = texture.clone();
        head_right.transformation =
            Matrix3::from_translation(vec2(0.8, 0.8)) *
                Matrix3::from_scale(0.2);

        let mut head_down = texture.clone();
        head_down.transformation =
            Matrix3::from_translation(vec2(0.8, 0.6)) *
                Matrix3::from_scale(0.2);

        let mut head_left = texture.clone();
        head_left.transformation =
            Matrix3::from_translation(vec2(0.6, 0.6)) *
                Matrix3::from_scale(0.2);

        let mut horizontal = texture.clone();
        horizontal.transformation =
            Matrix3::from_translation(vec2(0.2, 0.8)) *
                Matrix3::from_scale(0.2);

        let mut vertical = texture.clone();
        vertical.transformation =
            Matrix3::from_translation(vec2(0.4, 0.6)) *
                Matrix3::from_scale(0.2);

        Self {
            head_up,
            head_right,
            head_down,
            head_left,

            horizontal,
            vertical,
        }
    }

    fn material(texture: &Texture2DRef) -> ColorMaterial {
        ColorMaterial {
            texture: Some(texture.clone()),
            is_transparent: true,
            render_states: RenderStates {
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn head_up(&self) -> ColorMaterial {
        Self::material(&self.head_up)
    }

    pub fn head_right(&self) -> ColorMaterial {
        Self::material(&self.head_right)
    }

    pub fn head_down(&self) -> ColorMaterial {
        Self::material(&self.head_down)
    }

    pub fn head_left(&self) -> ColorMaterial {
        Self::material(&self.head_left)
    }

    pub fn horizontal(&self) -> ColorMaterial {
        Self::material(&self.horizontal)
    }

    pub fn vertical(&self) -> ColorMaterial {
        Self::material(&self.vertical)
    }
}
