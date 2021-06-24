use std::path::PathBuf;

use cairo;
use gtk::{gio, glib, prelude::*};
use rsvg::{CairoRenderer, Loader, SvgHandle};

use anyhow::anyhow;

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

pub fn create_tmp(filename: &str) -> anyhow::Result<PathBuf> {
    let mut temp_path = glib::user_cache_dir().join("app-icon-preview");
    std::fs::create_dir_all(&temp_path)?;
    temp_path.push(filename);
    Ok(temp_path)
}

pub fn render(handle: &SvgHandle, basename: &str, output_size: f64, dest: Option<PathBuf>) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
    let renderer = CairoRenderer::new(handle);
    let dest = dest.unwrap_or(create_tmp(&format!("#hicolor-{}-{}.svg", output_size, basename))?);

    let mut surface = cairo::SvgSurface::new(output_size, output_size, Some(dest.clone())).unwrap();
    surface.set_document_unit(cairo::SvgUnit::Px);
    let cr = cairo::Context::new(&surface)?;
    let dimensions = renderer.intrinsic_dimensions();
    let width = dimensions.width.ok_or(anyhow!("Failed to read size"))?.length;
    let height = dimensions.height.ok_or(anyhow!("Failed to read size"))?.length;

    renderer.render_layer(&cr, None, &cairo::Rectangle { x: 0.0, y: 0.0, width, height })?;

    Ok((gio::File::for_path(dest), surface))
}

pub fn render_by_id(handle: &SvgHandle, basename: &str, id: &str, output_size: f64, dest: Option<PathBuf>) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
    let dest = dest.unwrap_or(create_tmp(&format!("{}-{}-{}.svg", id, output_size, basename))?);

    if handle.has_element_with_id(id)? {
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

        return Ok((gio::File::for_path(dest), surface));
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
    context.set_source_surface(&overlay, 0.0, 0.0);

    let mask = source.create_similar(cairo::Content::Alpha, output_size as i32, output_size as i32).unwrap();
    let cr_mask = cairo::Context::new(&mask)?;
    cr_mask.set_source_surface(&source, 0.0, 0.0);
    cr_mask.paint();
    context.mask_surface(&mask, 0.0, 0.0);

    Ok(())
}

pub fn is_valid_app_id(app_id: &str) -> bool {
    app_id.contains('.') && !app_id.ends_with('.') && !app_id.starts_with('.')
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
