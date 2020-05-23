use gio::prelude::FileExt;
use glib::Cast;
use rsvg_internals::{Dpi, Handle, LoadOptions, SizeCallback};
use std::path::PathBuf;

pub fn create_tmp(filename: &str) -> anyhow::Result<PathBuf> {
    let mut temp_path = glib::get_user_cache_dir().unwrap().join("app-icon-preview");
    std::fs::create_dir_all(&temp_path)?;
    temp_path.push(filename);
    Ok(temp_path)
}

pub fn render(file: &gio::File, output_size: f64, dest: Option<PathBuf>) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
    let stream = file.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
    let handle = Handle::from_stream(&LoadOptions::new(None), &stream, gio::NONE_CANCELLABLE)?;

    let dest = dest.unwrap_or({ create_tmp(&format!("#hicolor-{}-{}", output_size, file.get_basename().unwrap().to_str().unwrap()))? });

    let mut surface = cairo::SvgSurface::new(output_size, output_size, Some(dest.clone())).unwrap();
    surface.set_document_unit(cairo::SvgUnit::Px);
    let cr = cairo::Context::new(&surface);
    let dimensions = handle.get_dimensions(Dpi::default(), &SizeCallback::default(), false)?;
    handle.render_layer(
        &cr,
        None,
        &cairo::Rectangle {
            x: 0.0,
            y: 0.0,
            width: f64::from(dimensions.width),
            height: f64::from(dimensions.height),
        },
        Dpi::default(),
        false,
    )?;

    Ok((gio::File::new_for_path(dest), surface))
}

pub fn render_by_id(file: &gio::File, id: &str, output_size: f64, dest: Option<PathBuf>) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
    let stream = file.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
    let mut handle = Handle::from_stream(&LoadOptions::new(None), &stream, gio::NONE_CANCELLABLE)?;

    let dest = dest.unwrap_or({ create_tmp(&format!("{}-{}-{}", id, output_size, file.get_basename().unwrap().to_str().unwrap()))? });

    handle.set_stylesheet("#layer3,#layer2 {visibility: hidden}")?;
    if handle.has_sub(id)? {
        let viewport = {
            let doc = handle.get_dimensions(Dpi::default(), &SizeCallback::default(), false)?;

            cairo::Rectangle {
                x: 0.0,
                y: 0.0,
                width: doc.width as f64,
                height: doc.height as f64,
            }
        };
        let (rect, _) = handle.get_geometry_for_layer(Some(&id), &viewport, Dpi::default(), false)?;

        let mut surface = cairo::SvgSurface::new(rect.width, rect.height, Some(dest.clone())).unwrap();
        surface.set_document_unit(cairo::SvgUnit::Px);
        let cr = cairo::Context::new(&surface);

        cr.scale(output_size / rect.width, output_size / rect.height);
        cr.translate(-rect.x, -rect.y);

        handle.render_cairo_sub(&cr, None, Dpi::default(), &SizeCallback::default(), false)?;

        return Ok((gio::File::new_for_path(dest), surface));
    }
    anyhow::bail!("failed")
}

pub fn get_overlay(output_size: f64) -> anyhow::Result<cairo::SvgSurface> {
    let stripes = gio::File::new_for_uri("resource:///org/gnome/design/AppIconPreview/templates/stripes.svg");
    let stream = stripes.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
    let handle = Handle::from_stream(&LoadOptions::new(None), &stream, gio::NONE_CANCELLABLE)?;

    let dimensions = handle.get_dimensions(Dpi::default(), &SizeCallback::default(), false)?;

    let surface = cairo::SvgSurface::new(output_size, output_size, None::<&std::path::Path>).unwrap();

    let context = cairo::Context::new(&surface);
    handle.render_layer(
        &context,
        None,
        &cairo::Rectangle {
            x: 0.0,
            y: 0.0,
            width: f64::from(dimensions.width),
            height: f64::from(dimensions.height),
        },
        Dpi::default(),
        false,
    )?;
    Ok(surface)
}

pub fn render_stripes(source: &cairo::SvgSurface, output_size: f64) -> anyhow::Result<()> {
    let context = cairo::Context::new(&source);

    let overlay = get_overlay(output_size)?;
    context.set_source_surface(&overlay, 0.0, 0.0);

    let mask = source.create_similar(cairo::Content::Alpha, output_size as i32, output_size as i32).unwrap();
    let cr_mask = cairo::Context::new(&mask);
    cr_mask.set_source_surface(&source, 0.0, 0.0);
    cr_mask.paint();
    context.mask_surface(&mask, 0.0, 0.0);

    Ok(())
}

pub fn is_valid_app_id(app_id: &str) -> bool {
    app_id.contains('.') && !app_id.ends_with('.') && !app_id.starts_with('.')
}

pub fn clean_svg(file: PathBuf) -> anyhow::Result<()> {
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
    let data = svgcleaner::cleaner::load_file(file.to_str().unwrap())?;

    let mut document = svgcleaner::cleaner::parse_data(&data, &svgcleaner::ParseOptions::default()).unwrap();
    let _ = svgcleaner::cleaner::clean_doc(&mut document, &options, &svgcleaner::WriteOptions::default());
    let mut buf = vec![];
    svgcleaner::cleaner::write_buffer(&document, &svgcleaner::WriteOptions::default(), &mut buf);

    svgcleaner::cleaner::save_file(&buf, file.to_str().unwrap())?;

    Ok(())
}
