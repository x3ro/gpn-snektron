use three_d::*;

pub struct SnakeTextures {
    head: Texture2DRef,
    body: Texture2DRef,
    tail: Texture2DRef,
    left_turn: Texture2DRef,
    right_turn: Texture2DRef,

    // tail_up: Texture2DRef,
    // tail_right: Texture2DRef,
    // tail_down: Texture2DRef,
    // tail_left: Texture2DRef,



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

        let mut head = texture.clone();
        head.transformation =
            Matrix3::from_translation(vec2(0.6, 0.8)) *
                Matrix3::from_scale(0.2);

        // let mut head_right = texture.clone();
        // head_right.transformation =
        //     Matrix3::from_translation(vec2(0.8, 0.8)) *
        //         Matrix3::from_scale(0.2);
        //
        // let mut head_down = texture.clone();
        // head_down.transformation =
        //     Matrix3::from_translation(vec2(0.8, 0.6)) *
        //         Matrix3::from_scale(0.2);
        //
        // let mut head_left = texture.clone();
        // head_left.transformation =
        //     Matrix3::from_translation(vec2(0.6, 0.6)) *
        //         Matrix3::from_scale(0.2);

        // let mut horizontal = texture.clone();
        // horizontal.transformation =
        //     Matrix3::from_translation(vec2(0.2, 0.8)) *
        //         Matrix3::from_scale(0.2);

        let mut body = texture.clone();
        body.transformation =
            Matrix3::from_translation(vec2(0.4, 0.6)) *
                Matrix3::from_scale(0.2);

        let mut tail = texture.clone();
        tail.transformation =
            Matrix3::from_translation(vec2(0.8, 0.2)) *
                Matrix3::from_scale(0.2);

        let mut right_turn = texture.clone();
        right_turn.transformation =
            Matrix3::from_translation(vec2(0.0, 0.8)) *
                Matrix3::from_scale(0.2);

        let mut left_turn = texture.clone();
        left_turn.transformation =
            Matrix3::from_translation(vec2(0.4, 0.8)) *
                Matrix3::from_scale(0.2);

        Self {
            head,
            body,
            tail,
            right_turn,
            left_turn,
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

    pub fn head(&self) -> ColorMaterial {
        Self::material(&self.head)
    }

    pub fn right_turn(&self) -> ColorMaterial {
        Self::material(&self.right_turn)
    }

    pub fn left_turn(&self) -> ColorMaterial {
        Self::material(&self.left_turn)
    }

    pub fn body(&self) -> ColorMaterial {
        Self::material(&self.body)
    }

    pub fn tail(&self) -> ColorMaterial {
        Self::material(&self.tail)
    }
}
