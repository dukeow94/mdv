(function (nativeDensity) {
  // Keep bitmap assets at the density used by --snapshot-scale 1. Drawing into
  // a canvas (without exporting it) also works for file:// and cross-origin
  // images: canvas export would fail their origin-clean checks.
  const resize = (source, width, height) => {
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("failed to prepare native-density image canvas");
    context.imageSmoothingEnabled = true;
    context.imageSmoothingQuality = "high";
    context.drawImage(source, 0, 0, width, height);
    return canvas;
  };
  const prepared = [];
  for (const image of Array.from(document.images)) {
    const bounds = image.getBoundingClientRect();
    if (!image.complete || !image.naturalWidth || !image.naturalHeight ||
        bounds.width <= 0 || bounds.height <= 0) continue;
    const style = getComputedStyle(image);
    const number = (property) => Number.parseFloat(style.getPropertyValue(property)) || 0;
    let width = number("width");
    let height = number("height");
    if (style.boxSizing === "border-box") {
      width -= number("padding-left") + number("padding-right") +
        number("border-left-width") + number("border-right-width");
      height -= number("padding-top") + number("padding-bottom") +
        number("border-top-width") + number("border-bottom-width");
    }
    if (width <= 0 || height <= 0) continue;

    // Preserve the source aspect ratio for object-fit. The canvas keeps the
    // original content-box size and object-position through its computed style.
    let rasterWidth = width;
    let rasterHeight = height;
    if (style.objectFit !== "fill") {
      const contain = Math.min(width / image.naturalWidth, height / image.naturalHeight);
      const cover = Math.max(width / image.naturalWidth, height / image.naturalHeight);
      const fit = style.objectFit === "cover" ? cover :
        style.objectFit === "none" ? 1 :
        style.objectFit === "scale-down" ? Math.min(1, contain) : contain;
      rasterWidth = image.naturalWidth * fit;
      rasterHeight = image.naturalHeight * fit;
    }
    const targetWidth = Math.max(1, Math.round(rasterWidth * nativeDensity));
    const targetHeight = Math.max(1, Math.round(rasterHeight * nativeDensity));
    let source = image;
    let sourceWidth = image.naturalWidth;
    let sourceHeight = image.naturalHeight;
    // WebKit's single-step canvas minification can alias a dithered GIF even
    // with smoothingQuality=high. Halving first supplies a low-pass prefilter.
    while (sourceWidth > targetWidth * 2 || sourceHeight > targetHeight * 2) {
      sourceWidth = sourceWidth > targetWidth * 2 ? Math.floor(sourceWidth / 2) : sourceWidth;
      sourceHeight = sourceHeight > targetHeight * 2 ? Math.floor(sourceHeight / 2) : sourceHeight;
      source = resize(source, sourceWidth, sourceHeight);
    }
    const canvas = resize(source, targetWidth, targetHeight);
    for (const attribute of image.attributes) {
      if (attribute.name !== "width" && attribute.name !== "height") {
        canvas.setAttribute(attribute.name, attribute.value);
      }
    }
    for (const property of style) {
      canvas.style.setProperty(property, style.getPropertyValue(property), "important");
    }
    // Used dimensions must not fall back to the canvas's new intrinsic density.
    canvas.style.setProperty("width", style.width, "important");
    canvas.style.setProperty("height", style.height, "important");
    prepared.push({ image, canvas, bounds });
  }
  // Prepare every source before changing the DOM so img-dependent selectors
  // cannot affect another image's captured computed style.
  for (const { image, canvas } of prepared) image.replaceWith(canvas);
  for (const { canvas, bounds } of prepared) {
    const actual = canvas.getBoundingClientRect();
    if (["x", "y", "width", "height"].some((key) => Math.abs(actual[key] - bounds[key]) > 0.1)) {
      throw new Error("native-density image preparation changed document layout");
    }
  }
  return prepared.length;
})
