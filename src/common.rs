use std::path::PathBuf;

use gtk::{gio, glib, prelude::*};
use rsvg::{CairoRenderer, Loader, SvgHandle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Icon {
    Symbolic,
    Scalable,
    Devel,
}

impl Default for Icon {
    fn default() -> Self {
        Self::Scalable
    }
}

impl<T: AsRef<str>> From<T> for Icon {
    fn from(name: T) -> Self {
        match name.as_ref() {
            "nightly" => Self::Devel,
            "regular" => Self::Scalable,
            "symbolic" => Self::Symbolic,
            _ => Self::default(),
        }
    }
}

impl Icon {
    pub fn path(self) -> PathBuf {
        match self {
            Icon::Symbolic | Icon::Scalable => icon_theme_path().join("hicolor/symbolic/apps"),
            Icon::Devel => icon_theme_path().join("hicolor/scalable/apps"),
        }
    }

    pub fn id(self) -> String {
        match self {
            Icon::Scalable | Icon::Devel => "#hicolor",
            Icon::Symbolic => "#symbolic",
        }
        .to_string()
    }

    pub fn size(self) -> f64 {
        match self {
            Icon::Scalable | Icon::Devel => 128.0,
            Icon::Symbolic => 16.0,
        }
    }
}

pub fn icon_theme_path() -> PathBuf {
    glib::user_cache_dir().join("app-icon-preview").join("icons")
}

pub fn format_name(name: &str) -> String {
    let name = name.trim_end_matches(".svg").trim_end_matches(".Source").split('.').last().unwrap();
    let mut formatted_chars = vec![];

    let mut chars = name.chars();
    name.chars().for_each(|c| {
        if c.is_uppercase() && !chars.next().unwrap_or(' ').is_uppercase() {
            formatted_chars.push(' ');
        }
        formatted_chars.push(c);
    });
    let formatted_name: String = formatted_chars.iter().collect();
    formatted_name
}

#[cfg(test)]
mod tests {
    use super::format_name;

    #[test]
    fn test_format_name() {
        assert_eq!(format_name("org.gnome.design.AppIconPreview"), "App Icon Preview".to_string());
        assert_eq!(format_name("org.gnome.GTG"), "GTG".to_string());
        assert_eq!(format_name("org.gnome.design.BannerViewer"), "Banner Viewer".to_string());
        assert_eq!(format_name("org.gnome.design.Contrast"), "Contrast".to_string());
    }
}

pub fn create_tmp(icon: Icon, icon_name: &str) -> anyhow::Result<PathBuf> {
    let mut temp_path = icon.path();
    std::fs::create_dir_all(&temp_path)?;
    let basename = match icon {
        Icon::Symbolic => format!("{}-symbolic.svg", icon_name),
        Icon::Scalable => format!("{}.svg", icon_name),
        Icon::Devel => format!("{}.Devel.svg", icon_name),
    };
    temp_path.push(&basename);
    Ok(temp_path)
}

pub fn init_tmp(icon_theme: &gtk::IconTheme) -> anyhow::Result<()> {
    // Symbolic dir: icons/hicolor/symbolic/apps
    // App icon dir: icons/hicolor/scalable/apps
    let temp_path = icon_theme_path();

    std::fs::create_dir_all(Icon::Scalable.path())?;
    std::fs::create_dir_all(Icon::Symbolic.path())?;

    icon_theme.add_search_path(&temp_path);
    Ok(())
}

pub fn render(handle: &SvgHandle, icon_name: &str, icon: Icon) -> anyhow::Result<()> {
    let output_size = icon.size();

    let renderer = CairoRenderer::new(handle);
    let dest = create_tmp(icon, icon_name)?;

    let mut surface = cairo::SvgSurface::new(output_size, output_size, Some(dest.clone())).unwrap();
    surface.set_document_unit(cairo::SvgUnit::Px);
    let cr = cairo::Context::new(&surface)?;
    let dimensions = renderer.intrinsic_dimensions();
    let width = dimensions.width.unwrap().length;
    let height = dimensions.height.unwrap().length;

    renderer.render_layer(&cr, None, &cairo::Rectangle { x: 0.0, y: 0.0, width, height })?;

    if icon == Icon::Devel {
        render_stripes(&surface, icon.size())?
    }

    Ok(())
}

pub fn render_by_id(handle: &SvgHandle, icon_name: &str, icon: Icon) -> anyhow::Result<()> {
    let dest = create_tmp(icon, icon_name)?;
    let id = icon.id();
    let output_size = icon.size();

    if handle.has_element_with_id(&id)? {
        let renderer = CairoRenderer::new(handle);
        let viewport = {
            let doc = renderer.intrinsic_dimensions();
            let width = doc.width.unwrap().length;
            let height = doc.height.unwrap().length;

            cairo::Rectangle { x: 0.0, y: 0.0, width, height }
        };
        let (rect, _) = renderer.geometry_for_layer(Some(&id), &viewport)?;

        let mut surface = cairo::SvgSurface::new(rect.width, rect.height, Some(dest.clone())).unwrap();
        surface.set_document_unit(cairo::SvgUnit::Px);
        let cr = cairo::Context::new(&surface)?;

        cr.scale(output_size / rect.width, output_size / rect.height);
        cr.translate(-rect.x, -rect.y);

        renderer.render_layer(&cr, None, &viewport)?;

        if icon == Icon::Devel {
            render_stripes(&surface, icon.size())?
        }

        return Ok(());
    }
    anyhow::bail!("failed")
}

pub fn get_overlay(output_size: f64) -> anyhow::Result<cairo::SvgSurface> {
    let stripes = gio::File::for_uri("resource:///org/gnome/design/AppIconPreview/templates/stripes.svg");
    let stream = stripes.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
    let handle = Loader::new().read_stream(&stream, Some(&stripes), gio::NONE_CANCELLABLE)?;

    let renderer = CairoRenderer::new(&handle);
    let dimensions = renderer.intrinsic_dimensions();

    let surface = cairo::SvgSurface::new(output_size, output_size, None::<&std::path::Path>).unwrap();

    let context = cairo::Context::new(&surface)?;
    let width = dimensions.width.unwrap().length;
    let height = dimensions.height.unwrap().length;

    renderer.render_layer(&context, None, &cairo::Rectangle { x: 0.0, y: 0.0, width, height })?;
    Ok(surface)
}

pub fn render_stripes(source: &cairo::SvgSurface, output_size: f64) -> anyhow::Result<()> {
    let context = cairo::Context::new(&source)?;

    let overlay = get_overlay(output_size)?;
    context.set_source_surface(&overlay, 0.0, 0.0)?;

    let mask = source.create_similar(cairo::Content::Alpha, output_size as i32, output_size as i32)?;
    let cr_mask = cairo::Context::new(&mask)?;
    cr_mask.set_source_surface(&source, 0.0, 0.0)?;
    cr_mask.paint()?;
    context.mask_surface(&mask, 0.0, 0.0)?;

    Ok(())
}

pub fn clean_svg(svg: &str) -> anyhow::Result<Vec<u8>> {
    let options = svgcleaner::CleaningOptions {
        remove_unused_defs: true,
        convert_shapes: false,
        remove_title: true,
        remove_desc: true,
        remove_metadata: true,
        remove_dupl_linear_gradients: true,
        remove_dupl_radial_gradients: true,
        remove_dupl_fe_gaussian_blur: true,
        ungroup_groups: true,
        ungroup_defs: true,
        group_by_style: true,
        merge_gradients: true,
        regroup_gradient_stops: true,
        remove_invalid_stops: true,
        remove_invisible_elements: true,
        resolve_use: true,
        remove_version: true,
        remove_unreferenced_ids: true,
        trim_ids: true,
        remove_text_attributes: true,
        remove_unused_coordinates: true,
        remove_default_attributes: true,
        remove_xmlns_xlink_attribute: true,
        remove_needless_attributes: true,
        remove_gradient_attributes: true,
        join_style_attributes: svgcleaner::StyleJoinMode::None,
        apply_transform_to_gradients: true,
        apply_transform_to_shapes: true,

        paths_to_relative: true,
        remove_unused_segments: true,
        convert_segments: true,
        apply_transform_to_paths: true,

        coordinates_precision: 6,
        properties_precision: 6,
        paths_coordinates_precision: 8,
        transforms_precision: 8,
    };
    let mut document = svgcleaner::cleaner::parse_data(svg, &svgcleaner::ParseOptions::default()).unwrap();
    let _ = svgcleaner::cleaner::clean_doc(&mut document, &options, &svgcleaner::WriteOptions::default());
    let mut buf = vec![];
    svgcleaner::cleaner::write_buffer(&document, &svgcleaner::WriteOptions::default(), &mut buf);

    Ok(buf)
}
