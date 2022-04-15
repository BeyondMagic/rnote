use anyhow::Context;
use gtk4::{gdk, glib, graphene, gsk, prelude::*, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};
use svg::node::element;

use crate::render;
use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::Color;

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "PatternStyle")]
#[serde(rename = "pattern_style")]
pub enum PatternStyle {
    #[enum_value(name = "None", nick = "none")]
    #[serde(rename = "none")]
    None = 0,
    #[enum_value(name = "Lines", nick = "lines")]
    #[serde(rename = "lines")]
    Lines,
    #[enum_value(name = "Grid", nick = "grid")]
    #[serde(rename = "grid")]
    Grid,
    #[enum_value(name = "Dots", nick = "dots")]
    #[serde(rename = "dots")]
    Dots,
}

impl Default for PatternStyle {
    fn default() -> Self {
        Self::Dots
    }
}

pub fn gen_hline_pattern(
    bounds: AABB,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::random_id_prefix() + "_bg_hline_pattern";

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", bounds.extents()[0])
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", 0_f64)
                    .set("x2", bounds.extents()[0])
                    .set("y2", 0_f64),
            ),
    );

    let rect = element::Rectangle::new()
        .set("x", bounds.mins[0])
        .set("y", bounds.mins[1])
        .set("width", bounds.extents()[0])
        .set("height", bounds.extents()[1])
        .set("fill", format!("url(#{})", pattern_id));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

pub fn gen_grid_pattern(
    bounds: AABB,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::random_id_prefix() + "_bg_grid_pattern";

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", column_spacing)
            .set("height", row_spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", 0_f64)
                    .set("x2", column_spacing)
                    .set("y2", 0_f64),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", 0_f64)
                    .set("x2", 0_f64)
                    .set("y2", row_spacing),
            ),
    );

    let rect = element::Rectangle::new()
        .set("x", bounds.mins[0])
        .set("y", bounds.mins[1])
        .set("width", bounds.extents()[0])
        .set("height", bounds.extents()[1])
        .set("fill", format!("url(#{})", pattern_id));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

pub fn gen_dots_pattern(
    bounds: AABB,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    dots_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::random_id_prefix() + "_bg_dots_pattern";

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", column_spacing)
            .set("height", row_spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Rectangle::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("x", 0_f64)
                    .set("y", 0_f64)
                    .set("width", dots_width)
                    .set("height", dots_width),
            ),
    );

    let rect = element::Rectangle::new()
        .set("x", bounds.mins[0])
        .set("y", bounds.mins[1])
        .set("width", bounds.extents()[0])
        .set("height", bounds.extents()[1])
        .set("fill", format!("url(#{})", pattern_id));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "background")]
pub struct Background {
    #[serde(rename = "color")]
    pub color: Color,
    #[serde(rename = "pattern")]
    pub pattern: PatternStyle,
    #[serde(rename = "pattern_size")]
    pub pattern_size: na::Vector2<f64>,
    #[serde(rename = "pattern_color")]
    pub pattern_color: Color,
    #[serde(skip)]
    pub image: Option<render::Image>,
    #[serde(skip)]
    rendernodes: Vec<gsk::RenderNode>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Self::COLOR_DEFAULT,
            pattern: PatternStyle::default(),
            pattern_size: Self::PATTERN_SIZE_DEFAULT,
            pattern_color: Self::PATTERN_COLOR_DEFAULT,
            image: None,
            rendernodes: vec![],
        }
    }
}

impl Background {
    pub const TILE_MAX_SIZE: f64 = 192.0;
    pub const COLOR_DEFAULT: Color = Color::WHITE;
    pub const PATTERN_SIZE_DEFAULT: na::Vector2<f64> = na::vector![32.0, 32.0];
    pub const PATTERN_COLOR_DEFAULT: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 1.0,
    };

    pub fn tile_size(&self) -> na::Vector2<f64> {
        // Calculate tile size as multiple of pattern_size with max size TITLE_MAX_SIZE
        let tile_factor =
            na::Vector2::from_element(Self::TILE_MAX_SIZE).component_div(&self.pattern_size);

        let tile_width = if tile_factor[0] > 1.0 {
            tile_factor[0].floor() * self.pattern_size[0]
        } else {
            self.pattern_size[0]
        };
        let tile_height = if tile_factor[1] > 1.0 {
            tile_factor[1].floor() * self.pattern_size[1]
        } else {
            self.pattern_size[1]
        };
        let tile_size = na::vector![tile_width, tile_height];

        tile_size
    }

    /// Generates the background svg, without xml header or svg root
    pub fn gen_svg(&self, bounds: AABB) -> Result<render::Svg, anyhow::Error> {
        let mut group = element::Group::new();

        // background color
        let color_rect = element::Rectangle::new()
            .set("x", bounds.mins[0])
            .set("y", bounds.mins[1])
            .set("width", bounds.extents()[0])
            .set("height", bounds.extents()[1])
            .set("fill", self.color.to_css_color_attr());
        group = group.add(color_rect);

        match self.pattern {
            PatternStyle::None => {}
            PatternStyle::Lines => {
                group = group.add(gen_hline_pattern(
                    bounds,
                    self.pattern_size[1],
                    self.pattern_color,
                    1.0,
                ));
            }
            PatternStyle::Grid => {
                group = group.add(gen_grid_pattern(
                    bounds,
                    self.pattern_size[1],
                    self.pattern_size[0],
                    self.pattern_color,
                    1.0,
                ));
            }
            PatternStyle::Dots => {
                group = group.add(gen_dots_pattern(
                    bounds,
                    self.pattern_size[1],
                    self.pattern_size[0],
                    self.pattern_color,
                    2.0,
                ));
            }
        }
        let svg_data = rnote_compose::utils::svg_node_to_string(&group)
            .map_err(|e| anyhow::anyhow!("node_to_string() failed for background, {}", e))?;

        Ok(render::Svg { svg_data, bounds })
    }

    fn gen_image(
        &self,
        bounds: AABB,
        image_scale: f64,
    ) -> Result<Option<render::Image>, anyhow::Error> {
        let svg = self.gen_svg(bounds)?;
        Ok(render::Image::join_images(
            render::Image::gen_images_from_svg(svg, bounds, image_scale)?,
            bounds,
            image_scale,
        )?)
    }

    fn gen_rendernodes(
        &mut self,
        sheet_bounds: AABB,
    ) -> Result<Vec<gsk::RenderNode>, anyhow::Error> {
        let tile_size = self.tile_size();
        let mut rendernodes: Vec<gsk::RenderNode> = vec![];

        // Fill with background color just in case there is any space left between the tiles
        rendernodes.push(
            gsk::ColorNode::new(
                &gdk::RGBA::from_compose_color(self.color),
                &graphene::Rect::from_aabb(sheet_bounds),
            )
            .upcast(),
        );

        if let Some(image) = &self.image {
            let new_texture = image
                .to_memtexture()
                .context("image_to_memtexture() failed in gen_rendernode().")?;
            for splitted_bounds in sheet_bounds.split_extended_origin_aligned(tile_size) {
                rendernodes.push(
                    gsk::TextureNode::new(
                        &new_texture,
                        &graphene::Rect::from_aabb(splitted_bounds.ceil().loosened(1.0)),
                    )
                    .upcast(),
                );
            }
        }

        Ok(rendernodes)
    }

    pub fn update_rendernodes(&mut self, sheet_bounds: AABB) -> anyhow::Result<()> {
        match self.gen_rendernodes(sheet_bounds) {
            Ok(rendernodes) => {
                self.rendernodes = rendernodes;
            }
            Err(e) => {
                log::error!(
                    "gen_rendernode() failed in update_rendernode() of background with Err: {}",
                    e
                );
            }
        }

        Ok(())
    }

    pub fn regenerate_background(
        &mut self,
        sheet_bounds: AABB,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        let tile_size = self.tile_size();
        let tile_bounds = AABB::new(na::point![0.0, 0.0], na::point![tile_size[0], tile_size[1]]);

        self.image = self.gen_image(tile_bounds, image_scale)?;

        self.update_rendernodes(sheet_bounds)?;
        Ok(())
    }

    pub fn draw(&self, snapshot: &Snapshot, sheet_bounds: AABB) {
        snapshot.push_clip(&graphene::Rect::from_aabb(sheet_bounds));

        self.rendernodes.iter().for_each(|rendernode| {
            snapshot.append_node(rendernode);
        });

        snapshot.pop();
    }
}