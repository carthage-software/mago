<?php

class Imagick
{
    public function optimizeImageLayers(): Imagick
    {
    }

    public function compareImageLayers(int $metric): Imagick
    {
    }

    public function pingImageBlob(string $image): bool
    {
    }

    public function pingImageFile(/*resource*/ mixed $filehandle, null|string $filename = null): bool
    {
    }

    public function transposeImage(): bool
    {
    }

    public function transverseImage(): bool
    {
    }

    public function trimImage(float $fuzz): bool
    {
    }

    public function waveImage(float $amplitude, float $length): bool
    {
    }

    public function waveImageWithMethod(float $amplitude, float $length, int $interpolate_method): bool // INTERPOLATE_*
    {
    }

    public function vignetteImage(float $black_point, float $white_point, int $x, int $y): bool
    {
    }

    public function uniqueImageColors(): bool
    {
    }

    /** @deprecated */
    public function getImageMatte(): bool
    {
    }

    public function setImageMatte(bool $matte): bool
    {
    }

    public function adaptiveResizeImage(int $columns, int $rows, bool $bestfit = false, bool $legacy = false): bool
    {
    }

    public function sketchImage(float $radius, float $sigma, float $angle): bool
    {
    }

    public function shadeImage(bool $gray, float $azimuth, float $elevation): bool
    {
    }

    public function getSizeOffset(): int
    {
    }

    public function setSizeOffset(int $columns, int $rows, int $offset): bool
    {
    }

    public function adaptiveBlurImage(float $radius, float $sigma, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function contrastStretchImage(
        float $black_point,
        float $white_point,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function adaptiveSharpenImage(float $radius, float $sigma, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function randomThresholdImage(float $low, float $high, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function roundCornersImage(
        float $x_rounding,
        float $y_rounding,
        float $stroke_width = 10,
        float $displace = 5,
        float $size_correction = -6,
    ): bool {
    }

    public function roundCorners(
        float $x_rounding,
        float $y_rounding,
        float $stroke_width = 10,
        float $displace = 5,
        float $size_correction = -6,
    ): bool {
    }

    public function setIteratorIndex(int $index): bool
    {
    }

    public function getIteratorIndex(): int
    {
    }

    /** @deprecated */
    public function transformImage(string $crop, string $geometry): Imagick
    {
    }

    /** @deprecated */
    public function setImageOpacity(float $opacity): bool
    {
    }

    public function setImageAlpha(float $alpha): bool
    {
    }

    /** @deprecated */
    public function orderedPosterizeImage(string $threshold_map, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function polaroidWithTextAndMethod(ImagickDraw $settings, float $angle, string $caption, int $method): bool
    {
    }

    public function polaroidImage(ImagickDraw $settings, float $angle): bool
    {
    }

    public function getImageProperty(string $name): string
    {
    }

    public function setImageProperty(string $name, string $value): bool
    {
    }

    public function deleteImageProperty(string $name): bool
    {
    }

    public function identifyFormat(string $format): string
    {
    }

    public function setImageInterpolateMethod(int $method): bool
    {
    }

    public function getImageInterpolateMethod(): int
    {
    }

    public function linearStretchImage(float $black_point, float $white_point): bool
    {
    }

    public function getImageLength(): int
    {
    }

    public function extentImage(int $width, int $height, int $x, int $y): bool
    {
    }

    public function getImageOrientation(): int
    {
    }

    public function setImageOrientation(int $orientation): bool
    {
    }

    /** @deprecated */
    public function paintFloodfillImage(
        ImagickPixel|string $fill_color,
        float $fuzz,
        ImagickPixel|string $border_color,
        int $x,
        int $y,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function clutImage(Imagick $lookup_table, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function clutImageWithInterpolate(Imagick $lookup_table, int $pixel_interpolate_method): bool // PixelInterpolateMethod
    {
    }

    public function getImageProperties(string $pattern = '*', bool $include_values = true): array
    {
    }

    public function getImageProfiles(string $pattern = '*', bool $include_values = true): array
    {
    }

    public function distortImage(int $distortion, array $arguments, bool $bestfit): bool
    {
    }

    public function writeImageFile(/*resource*/ mixed $filehandle, null|string $format = null): bool
    {
    }

    public function writeImagesFile(/*resource*/ mixed $filehandle, null|string $format = null): bool
    {
    }

    public function resetImagePage(string $page): bool
    {
    }

    /** @deprecated */
    public function setImageClipMask(imagick $clip_mask): bool
    {
    }

    /** @deprecated */
    public function getImageClipMask(): Imagick
    {
    }

    public function animateImages(string $x_server): bool
    {
    }

    /** @deprecated */
    public function recolorImage(array $matrix): bool
    {
    }

    public function setFont(string $font): bool
    {
    }

    public function getFont(): string
    {
    }

    public function setPointSize(float $point_size): bool
    {
    }

    public function getPointSize(): float
    {
    }

    public function mergeImageLayers(int $layermethod): Imagick
    {
    }

    public function setImageAlphaChannel(int $alphachannel): bool
    {
    }

    public function floodfillPaintImage(
        ImagickPixel|string $fill_color,
        float $fuzz,
        ImagickPixel|string $border_color,
        int $x,
        int $y,
        bool $invert,
        null|int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function opaquePaintImage(
        ImagickPixel|string $target_color,
        ImagickPixel|string $fill_color,
        float $fuzz,
        bool $invert,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function transparentPaintImage(
        ImagickPixel|string $target_color,
        float $alpha,
        float $fuzz,
        bool $invert,
    ): bool {
    }

    public function liquidRescaleImage(int $width, int $height, float $delta_x, float $rigidity): bool
    {
    }

    public function encipherImage(string $passphrase): bool
    {
    }

    public function decipherImage(string $passphrase): bool
    {
    }

    public function setGravity(int $gravity): bool
    {
    }

    public function getGravity(): int
    {
    }

    public function getImageChannelRange(int $channel): array
    {
    }

    public function getImageAlphaChannel(): bool
    {
    }

    public function getImageChannelDistortions(
        Imagick $reference_image,
        int $metric,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): float {
    }

    public function setImageGravity(int $gravity): bool
    {
    }

    public function getImageGravity(): int
    {
    }

    public function importImagePixels(
        int $x,
        int $y,
        int $width,
        int $height,
        string $map,
        int $pixelstorage,
        array $pixels,
    ): bool { // PIXELSTORAGE
    }

    public function deskewImage(float $threshold): bool
    {
    }

    public function segmentImage(
        int $colorspace,
        float $cluster_threshold,
        float $smooth_threshold,
        bool $verbose = false,
    ): bool { // COLORSPACE
    }

    public function sparseColorImage(
        int $sparsecolormethod,
        array $arguments,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool { // SPARSECOLORMETHOD_*
    }

    public function remapImage(Imagick $replacement, int $dither_method): bool
    {
    }

    public function houghLineImage(int $width, int $height, float $threshold): bool
    {
    }

    public function exportImagePixels(int $x, int $y, int $width, int $height, string $map, int $pixelstorage): array // e.g. "RGB" // PIXELSTORAGE
    {
    }

    public function getImageChannelKurtosis(int $channel = Imagick::CHANNEL_DEFAULT): array
    {
    }

    public function functionImage(int $function, array $parameters, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function transformImageColorspace(int $colorspace): bool
    {
    }

    public function haldClutImage(Imagick $clut, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function autoLevelImage(int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function blueShiftImage(float $factor = 1.5): bool
    {
    }

    public function getImageArtifact(string $artifact): string|null
    {
    }

    public function setImageArtifact(string $artifact, string|null $value): bool
    {
    }

    public function deleteImageArtifact(string $artifact): bool
    {
    }

    public function getColorspace(): int
    {
    }

    public function setColorspace(int $colorspace): bool
    {
    }

    public function clampImage(int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function smushImages(bool $stack, int $offset): Imagick
    {
    }

    public function __construct(string|array|int|float|null $files = null) {}

    public function __toString(): string
    {
    }

    public function count(int $mode = 0): int
    {
    }

    public function count(): int
    {
    }

    public function getPixelIterator(): ImagickPixelIterator
    {
    }

    public function getPixelRegionIterator(int $x, int $y, int $columns, int $rows): ImagickPixelIterator
    {
    }

    public function readImage(string $filename): bool
    {
    }

    public function readImages(array $filenames): bool
    {
    }

    public function readImageBlob(string $image, null|string $filename = null): bool
    {
    }

    public function setImageFormat(string $format): bool
    {
    }

    public function scaleImage(int $columns, int $rows, bool $bestfit = false, bool $legacy = false): bool
    {
    }

    public function writeImage(null|string $filename = null): bool
    {
    }

    public function writeImages(string $filename, bool $adjoin): bool
    {
    }

    public function blurImage(float $radius, float $sigma, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function thumbnailImage(
        null|int $columns,
        null|int $rows,
        bool $bestfit = false,
        bool $fill = false,
        bool $legacy = false,
    ): bool {
    }

    public function cropThumbnailImage(int $width, int $height, bool $legacy = false): bool
    {
    }

    public function getImageFilename(): string
    {
    }

    public function setImageFilename(string $filename): bool
    {
    }

    public function getImageFormat(): string
    {
    }

    public function getImageMimeType(): string
    {
    }

    public function removeImage(): bool
    {
    }

    public function destroy(): bool
    {
    }

    public function clear(): bool
    {
    }

    public function clone(): Imagick
    {
    }

    public function getImageSize(): int
    {
    }

    public function getImageBlob(): string
    {
    }

    public function getImagesBlob(): string
    {
    }

    public function setFirstIterator(): bool
    {
    }

    public function setLastIterator(): bool
    {
    }

    public function resetIterator(): void
    {
    }

    public function previousImage(): bool
    {
    }

    public function nextImage(): bool
    {
    }

    public function hasPreviousImage(): bool
    {
    }

    public function hasNextImage(): bool
    {
    }

    public function setImageIndex(int $index): bool
    {
    }

    public function getImageIndex(): int
    {
    }

    public function commentImage(string $comment): bool
    {
    }

    public function cropImage(int $width, int $height, int $x, int $y): bool
    {
    }

    public function labelImage(string $label): bool
    {
    }

    public function getImageGeometry(): array
    {
    }

    public function drawImage(ImagickDraw $drawing): bool
    {
    }

    public function setImageCompressionQuality(int $quality): bool
    {
    }

    public function getImageCompressionQuality(): int
    {
    }

    public function setImageCompression(int $compression): bool
    {
    }

    public function getImageCompression(): int
    {
    }

    public function annotateImage(ImagickDraw $settings, float $x, float $y, float $angle, string $text): bool
    {
    }

    public function compositeImage(
        Imagick $composite_image,
        int $composite,
        int $x,
        int $y,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function modulateImage(float $brightness, float $saturation, float $hue): bool
    {
    }

    public function getImageColors(): int
    {
    }

    public function montageImage(
        ImagickDraw $settings,
        string $tile_geometry,
        string $thumbnail_geometry,
        int $monatgemode,
        string $frame,
    ): Imagick { // e.g. "3x2+0+0" // e.g. "200x160+3+3>" // MONTAGEMODE_ // "10x10+2+2"
    }

    public function identifyImage(bool $append_raw_output = false): array
    {
    }

    public function thresholdImage(float $threshold, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function adaptiveThresholdImage(int $width, int $height, int $offset): bool
    {
    }

    public function blackThresholdImage(ImagickPixel|string $threshold_color): bool
    {
    }

    public function whiteThresholdImage(ImagickPixel|string $threshold_color): bool
    {
    }

    public function appendImages(bool $stack): Imagick
    {
    }

    public function charcoalImage(float $radius, float $sigma): bool
    {
    }

    public function normalizeImage(int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function oilPaintImageWithSigma(float $radius, float $sigma): bool
    {
    }

    public function oilPaintImage(float $radius): bool
    {
    }

    public function posterizeImage(int $levels, bool $dither): bool
    {
    }

    /** @deprecated */
    public function radialBlurImage(float $angle, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function raiseImage(int $width, int $height, int $x, int $y, bool $raise): bool
    {
    }

    public function resampleImage(float $x_resolution, float $y_resolution, int $filter, float $blur): bool
    {
    }

    public function resizeImage(
        int $columns,
        int $rows,
        int $filter,
        float $blur,
        bool $bestfit = false,
        bool $legacy = false,
    ): bool {
    }

    public function rollImage(int $x, int $y): bool
    {
    }

    public function rotateImage(ImagickPixel|string $background_color, float $degrees): bool
    {
    }

    public function sampleImage(int $columns, int $rows): bool
    {
    }

    public function solarizeImage(int $threshold): bool
    {
    }

    public function shadowImage(float $opacity, float $sigma, int $x, int $y): bool
    {
    }

    /** @deprecated */
    public function setImageAttribute(string $key, string $value): bool
    {
    }

    public function setImageBackgroundColor(ImagickPixel|string $background_color): bool
    {
    }

    public function setImageChannelMask(int $channel): int
    {
    }

    public function setImageCompose(int $compose): bool
    {
    }

    public function setImageDelay(int $delay): bool
    {
    }

    public function setImageDepth(int $depth): bool
    {
    }

    public function setImageGamma(float $gamma): bool
    {
    }

    public function setImageIterations(int $iterations): bool
    {
    }

    public function setImageMatteColor(ImagickPixel|string $matte_color): bool
    {
    }

    public function setImagePage(int $width, int $height, int $x, int $y): bool
    {
    }

    public function setImageProgressMonitor(string $filename): bool
    {
    }

    public function setProgressMonitor(callable $callback): bool
    {
    }

    public function setImageResolution(float $x_resolution, float $y_resolution): bool
    {
    }

    public function setImageScene(int $scene): bool
    {
    }

    public function setImageTicksPerSecond(int $ticks_per_second): bool
    {
    }

    public function setImageType(int $image_type): bool
    {
    }

    public function setImageUnits(int $units): bool
    {
    }

    public function sharpenImage(float $radius, float $sigma, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function shaveImage(int $columns, int $rows): bool
    {
    }

    public function shearImage(ImagickPixel|string $background_color, float $x_shear, float $y_shear): bool
    {
    }

    public function spliceImage(int $width, int $height, int $x, int $y): bool
    {
    }

    public function pingImage(string $filename): bool
    {
    }

    public function readImageFile(/*resource*/ mixed $filehandle, null|string $filename = null): bool
    {
    }

    public function displayImage(string $servername): bool
    {
    }

    public function displayImages(string $servername): bool
    {
    }

    public function spreadImage(float $radius): bool
    {
    }

    public function spreadImageWithMethod(float $radius, int $interpolate_method): bool // INTERPOLATE_*
    {
    }

    public function swirlImage(float $degrees): bool
    {
    }

    public function swirlImageWithMethod(float $degrees, int $interpolate_method): bool // INTERPOLATE_*
    {
    }

    public function stripImage(): bool
    {
    }

    public static function queryFormats(string $pattern = '*'): array
    {
    }

    public static function queryFonts(string $pattern = '*'): array
    {
    }

    /* TODO  $multiline == null,  means we should autodetect */
    public function queryFontMetrics(ImagickDraw $settings, string $text, null|bool $multiline = null): array
    {
    }

    public function steganoImage(Imagick $watermark, int $offset): Imagick
    {
    }

    public function addNoiseImage(int $noise, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function addNoiseImageWithAttenuate(
        int $noise,
        float $attenuate,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function motionBlurImage(
        float $radius,
        float $sigma,
        float $angle,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    /** @deprecated */
    public function mosaicImages(): Imagick
    {
    }

    public function morphImages(int $number_frames): Imagick
    {
    }

    public function minifyImage(): bool
    {
    }

    public function affineTransformImage(ImagickDraw $settings): bool
    {
    }

    public function averageImages(): Imagick
    {
    }

    public function borderImage(ImagickPixel|string $border_color, int $width, int $height): bool
    {
    }

    public function borderImageWithComposite(
        ImagickPixel|string $border_color,
        int $width,
        int $height,
        int $composite,
    ): bool { // COMPOSITE_ // null rather than OverCompositeOp as we don't control the value
    }

    public static function calculateCrop(
        int $original_width,
        int $original_height,
        int $desired_width,
        int $desired_height,
        bool $legacy = false,
    ): array {
    }

    public function chopImage(int $width, int $height, int $x, int $y): bool
    {
    }

    public function clipImage(): bool
    {
    }

    public function clipPathImage(string $pathname, bool $inside): bool
    {
    }

    public function clipImagePath(string $pathname, bool $inside): void
    {
    }

    public function coalesceImages(): Imagick
    {
    }

    /** @deprecated */
    public function colorFloodfillImage(
        ImagickPixel|string $fill_color,
        float $fuzz,
        ImagickPixel|string $border_color,
        int $x,
        int $y,
    ): bool {
    }

    public function colorizeImage(
        ImagickPixel|string $colorize_color,
        ImagickPixel|string|false $opacity_color,
        null|bool $legacy = false,
    ): bool {
    }

    public function compareImageChannels(Imagick $reference, int $channel, int $metric): array
    {
    }

    public function compareImages(Imagick $reference, int $metric): array
    {
    }

    public function contrastImage(bool $sharpen): bool
    {
    }

    public function combineImages(int $colorspace): Imagick
    {
    }

    public function convolveImage(ImagickKernel $kernel, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function convolveImage(array $kernel, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function cycleColormapImage(int $displace): bool
    {
    }

    public function deconstructImages(): Imagick
    {
    }

    public function despeckleImage(): bool
    {
    }

    public function edgeImage(float $radius): bool
    {
    }

    public function embossImage(float $radius, float $sigma): bool
    {
    }

    public function enhanceImage(): bool
    {
    }

    public function equalizeImage(): bool
    {
    }

    public function evaluateImage(int $evaluate, float $constant, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function evaluateImages(int $evaluate): Imagick
    {
    }

    public function flattenImages(): Imagick
    {
    }

    public function flipImage(): bool
    {
    }

    public function flopImage(): bool
    {
    }

    public function forwardFourierTransformImage(bool $magnitude): bool
    {
    }

    public function frameImage(
        ImagickPixel|string $matte_color,
        int $width,
        int $height,
        int $inner_bevel,
        int $outer_bevel,
    ): bool {
    }

    public function frameImageWithComposite(
        ImagickPixel|string $matte_color,
        int $width,
        int $height,
        int $inner_bevel,
        int $outer_bevel,
        int $composite,
    ): bool {
    }

    public function fxImage(string $expression, int $channel = Imagick::CHANNEL_DEFAULT): Imagick
    {
    }

    public function gammaImage(float $gamma, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function gaussianBlurImage(float $radius, float $sigma, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    /** @deprecated */
    public function getImageAttribute(string $key): string
    {
    }

    public function getImageBackgroundColor(): ImagickPixel
    {
    }

    public function getImageBluePrimary(): array
    {
    }

    public function getImageBorderColor(): ImagickPixel
    {
    }

    public function getImageChannelDepth(int $channel): int
    {
    }

    public function getImageChannelDistortion(Imagick $reference, int $channel, int $metric): float
    {
    }

    /** @deprecated */
    public function getImageChannelExtrema(int $channel): array
    {
    }

    public function getImageChannelMean(int $channel): array
    {
    }

    public function getImageChannelStatistics(): array
    {
    }

    public function getImageColormapColor(int $index): ImagickPixel
    {
    }

    public function getImageColorspace(): int
    {
    }

    public function getImageCompose(): int
    {
    }

    public function getImageDelay(): int
    {
    }

    public function getImageDepth(): int
    {
    }

    public function getImageDistortion(Imagick $reference, int $metric): float
    {
    }

    /** @deprecated */
    public function getImageExtrema(): array
    {
    }

    public function getImageDispose(): int
    {
    }

    public function getImageGamma(): float
    {
    }

    public function getImageGreenPrimary(): array
    {
    }

    public function getImageHeight(): int
    {
    }

    public function getImageHistogram(): array
    {
    }

    public function getImageInterlaceScheme(): int
    {
    }

    public function getImageIterations(): int
    {
    }

    /** @deprecated */
    public function getImageMatteColor(): ImagickPixel
    {
    }

    public function getImagePage(): array
    {
    }

    public function getImagePixelColor(int $x, int $y): ImagickPixel
    {
    }

    public function setImagePixelColor(int $x, int $y, ImagickPixel|string $color): ImagickPixel
    {
    }

    public function getImageProfile(string $name): string
    {
    }

    public function getImageRedPrimary(): array
    {
    }

    public function getImageRenderingIntent(): int
    {
    }

    public function getImageResolution(): array
    {
    }

    public function getImageScene(): int
    {
    }

    public function getImageSignature(): string
    {
    }

    public function getImageTicksPerSecond(): int
    {
    }

    public function getImageType(): int
    {
    }

    public function getImageUnits(): int
    {
    }

    public function getImageVirtualPixelMethod(): int
    {
    }

    public function getImageWhitePoint(): array
    {
    }

    public function getImageWidth(): int
    {
    }

    public function getNumberImages(): int
    {
    }

    public function getImageTotalInkDensity(): float
    {
    }

    public function getImageRegion(int $width, int $height, int $x, int $y): Imagick
    {
    }

    public function implodeImage(float $radius): bool
    {
    }

    public function implodeImageWithMethod(float $radius, int $pixel_interpolate_method): bool // PixelInterpolateMethod
    {
    }

    public function inverseFourierTransformImage(Imagick $complement, bool $magnitude): bool
    {
    }

    public function levelImage(
        float $black_point,
        float $gamma,
        float $white_point,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function magnifyImage(): bool
    {
    }

    /** @deprecated */
    public function mapImage(imagick $map, bool $dither): bool
    {
    }

    /** @deprecated */
    public function matteFloodfillImage(
        float $alpha,
        float $fuzz,
        ImagickPixel|string $border_color,
        int $x,
        int $y,
    ): bool {
    }

    /** @deprecated */
    public function medianFilterImage(float $radius): bool
    {
    }

    public function negateImage(bool $gray, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    /** @deprecated */
    public function paintOpaqueImage(
        ImagickPixel|string $target_color,
        ImagickPixel|string $fill_color,
        float $fuzz,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    /** @deprecated */
    public function paintTransparentImage(ImagickPixel|string $target_color, float $alpha, float $fuzz): bool
    {
    }

    public function previewImages(int $preview): bool
    {
    }

    public function profileImage(string $name, null|string $profile): bool
    {
    }

    public function quantizeImage(
        int $number_colors,
        int $colorspace,
        int $tree_depth,
        bool $dither,
        bool $measure_error,
    ): bool {
    }

    public function quantizeImages(
        int $number_colors,
        int $colorspace,
        int $tree_depth,
        bool $dither,
        bool $measure_error,
    ): bool {
    }

    /** @deprecated */
    public function reduceNoiseImage(float $radius): bool
    {
    }

    public function removeImageProfile(string $name): string
    {
    }

    public function separateImageChannel(int $channel): bool
    {
    }

    public function sepiaToneImage(float $threshold): bool
    {
    }

    /** @deprecated */
    public function setImageBias(float $bias): bool
    {
    }

    /** @deprecated */
    public function setImageBiasQuantum(string $bias): void
    {
    }

    public function setImageBluePrimary(float $x, float $y, float $z): bool
    {
    }

    public function setImageBluePrimary(float $x, float $y): bool
    {
    }

    /* {{{ proto bool Imagick::setImageBluePrimary(float x,float y)
     * For IM7 the prototype is
     * proto bool Imagick::setImageBluePrimary(float x, float y, float z) */

    public function setImageBorderColor(ImagickPixel|string $border_color): bool
    {
    }

    public function setImageChannelDepth(int $channel, int $depth): bool
    {
    }

    public function setImageColormapColor(int $index, ImagickPixel|string $color): bool
    {
    }

    public function setImageColorspace(int $colorspace): bool
    {
    }

    public function setImageDispose(int $dispose): bool
    {
    }

    public function setImageExtent(int $columns, int $rows): bool
    {
    }

    public function setImageGreenPrimary(float $x, float $y, float $z): bool
    {
    }

    public function setImageGreenPrimary(float $x, float $y): bool
    {
    }

    public function setImageInterlaceScheme(int $interlace): bool
    {
    }

    public function setImageProfile(string $name, string $profile): bool
    {
    }

    public function setImageRedPrimary(float $x, float $y, float $z): bool
    {
    }

    public function setImageRedPrimary(float $x, float $y): bool
    {
    }

    public function setImageRenderingIntent(int $rendering_intent): bool
    {
    }

    public function setImageVirtualPixelMethod(int $method): bool
    {
    }

    public function setImageWhitePoint(float $x, float $y, float $z): bool
    {
    }

    public function setImageWhitePoint(float $x, float $y): bool
    {
    }

    public function sigmoidalContrastImage(
        bool $sharpen,
        float $alpha,
        float $beta,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function stereoImage(Imagick $offset_image): bool
    {
    }

    public function textureImage(Imagick $texture): Imagick
    {
    }

    public function tintImage(
        ImagickPixel|string $tint_color,
        ImagickPixel|string $opacity_color,
        bool $legacy = false,
    ): bool {
    }

    public function unsharpMaskImage(
        float $radius,
        float $sigma,
        float $amount,
        float $threshold,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function getImage(): Imagick
    {
    }

    public function addImage(Imagick $image): bool
    {
    }

    public function setImage(Imagick $image): bool
    {
    }

    public function newImage(
        int $columns,
        int $rows,
        ImagickPixel|string $background_color,
        null|string $format = null,
    ): bool {
    }

    public function newPseudoImage(int $columns, int $rows, string $pseudo_format): bool
    {
    }

    public function getCompression(): int
    {
    }

    public function getCompressionQuality(): int
    {
    }

    public static function getCopyright(): string
    {
    }

    /**
     * @return string[]
     */
    public static function getConfigureOptions(string $pattern = '*'): array
    {
    }

    public static function getFeatures(): string
    {
    }

    public function getFilename(): string
    {
    }

    public function getFormat(): string
    {
    }

    public static function getHomeURL(): string
    {
    }

    public function getInterlaceScheme(): int
    {
    }

    public function getOption(string $key): string
    {
    }

    public static function getPackageName(): string
    {
    }

    public function getPage(): array
    {
    }

    public static function getQuantum(): int
    {
    }

    public static function getHdriEnabled(): bool
    {
    }

    public static function getQuantumDepth(): array
    {
    }

    public static function getQuantumRange(): array
    {
    }

    public static function getReleaseDate(): string
    {
    }

    public static function getResource(int $type): int
    {
    }

    public static function getResourceLimit(int $type): float
    {
    }

    public function getSamplingFactors(): array
    {
    }

    public function getSize(): array
    {
    }

    public static function getVersion(): array
    {
    }

    public function setBackgroundColor(ImagickPixel|string $background_color): bool
    {
    }

    public function setCompression(int $compression): bool
    {
    }

    public function setCompressionQuality(int $quality): bool
    {
    }

    public function setFilename(string $filename): bool
    {
    }

    public function setFormat(string $format): bool
    {
    }

    public function setInterlaceScheme(int $interlace): bool
    {
    }

    public function setOption(string $key, string $value): bool
    {
    }

    public function setPage(int $width, int $height, int $x, int $y): bool
    {
    }

    public static function setResourceLimit(int $type, int $limit): bool
    {
    }

    public function setResolution(float $x_resolution, float $y_resolution): bool
    {
    }

    public function setSamplingFactors(array $factors): bool
    {
    }

    public function setSize(int $columns, int $rows): bool
    {
    }

    public function setType(int $imgtype): bool
    {
    }

    public function key(): int
    {
    }

    public function next(): void
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
    }

    public function current(): Imagick
    {
    }

    public function brightnessContrastImage(
        float $brightness,
        float $contrast,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function colorMatrixImage(array $color_matrix): bool
    {
    }

    public function selectiveBlurImage(
        float $radius,
        float $sigma,
        float $threshold,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    public function rotationalBlurImage(float $angle, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    public function statisticImage(int $type, int $width, int $height, int $channel = Imagick::CHANNEL_DEFAULT): bool
    {
    }

    /**
     * @param array $offset
     * @param float $similarity
     */
    public function subimageMatch(
        Imagick $image,
        &$offset = null,
        &$similarity = null,
        float $threshold = 0.0,
        int $metric = 0,
    ): Imagick {
    }

    /**
     * @param array $offset
     * @param float $similarity
     */
    public function similarityImage(
        Imagick $image,
        &$offset = null,
        &$similarity = null,
        float $threshold = 0.0,
        int $metric = 0,
    ): Imagick {
    }

    public static function setRegistry(string $key, string $value): bool
    {
    }

    public static function getRegistry(string $key): string|false
    {
    }

    public static function listRegistry(): array
    {
    }

    public function morphology(
        int $morphology,
        int $iterations,
        ImagickKernel $kernel,
        int $channel = Imagick::CHANNEL_DEFAULT,
    ): bool {
    }

    /** @deprecated */
    public function filter(ImagickKernel $kernel, int $channel = Imagick::CHANNEL_UNDEFINED): bool
    {
    }

    public function setAntialias(bool $antialias): void
    {
    }

    public function getAntialias(): bool
    {
    }

    public function colorDecisionListImage(string $color_correction_collection): bool
    {
    }

    public function optimizeImageTransparency(): void
    {
    }

    public function autoGammaImage(null|int $channel = Imagick::CHANNEL_ALL): void
    {
    }

    public function autoOrient(): void
    {
    }

    public function autoOrientate(): void
    {
    }

    public function compositeImageGravity(Imagick $image, int $composite_constant, int $gravity): bool
    {
    }

    public function localContrastImage(float $radius, float $strength): bool
    {
    }

    public function identifyImageType(): int
    {
    }

    public function getImageMask(int $pixelmask): null|Imagick
    {
    }

    public function setImageMask(Imagick $clip_mask, int $pixelmask): void
    {
    }

    public function cannyEdgeImage(float $radius, float $sigma, float $lower_percent, float $upper_percent): bool
    {
    }

    public static function setSeed(int $seed): void
    {
    }

    public function waveletDenoiseImage(float $threshold, float $softness): bool
    {
    }

    public function meanShiftImage(int $width, int $height, float $color_distance): bool
    {
    }

    public function kmeansImage(int $number_colors, int $max_iterations, float $tolerance): bool
    {
    }

    public function rangeThresholdImage(float $low_black, float $low_white, float $high_white, float $high_black): bool
    {
    }

    public function autoThresholdImage(int $auto_threshold_method): bool
    {
    }

    public function bilateralBlurImage(float $radius, float $sigma, float $intensity_sigma, float $spatial_sigma): bool
    {
    }

    public function claheImage(int $width, int $height, int $number_bins, float $clip_limit): bool
    {
    }

    public function channelFxImage(string $expression): Imagick
    {
    }

    public function colorThresholdImage(ImagickPixel|string $start_color, ImagickPixel|string $stop_color): bool
    {
    }

    public function complexImages(int $complex_operator): Imagick
    {
    }

    public function interpolativeResizeImage(int $columns, int $rows, int $interpolate): bool // INTERPOLATE_
    {
    }

    public function levelImageColors(
        ImagickPixel|string $black_color,
        ImagickPixel|string $white_color,
        bool $invert,
    ): bool {
    }

    public function levelizeImage(float $black_point, float $gamma, float $white_point): bool
    {
    }

    public function orderedDitherImage(string $dither_format): bool
    {
    }

    public function whiteBalanceImage(): bool
    {
    }

    public function deleteOption(string $option): bool
    {
    }

    public function getBackgroundColor(): ImagickPixel
    {
    }

    /**
     * @return string[]
     */
    public function getImageArtifacts(string $pattern = '*'): array
    {
    }

    /**
     * @return array{kurtosis: float, skewness: float}
     */
    public function getImageKurtosis(): array
    {
    }

    public function getImageMean(): array
    {
    }

    public function getImageRange(): array
    {
    }

    public function getInterpolateMethod(): int
    {
    }

    /**
     * @return string[]
     */
    public function getOptions(string $pattern = '*'): array
    {
    }

    public function getOrientation(): int
    {
    }

    /**
     * @return array{x: float, y: float}
     */
    public function getResolution(): array
    {
    }

    public function getType(): int
    {
    }

    public function polynomialImage(array $terms): bool
    {
    }

    public function setDepth(int $depth): bool
    {
    }

    public function setExtract(string $geometry): bool
    {
    }

    public function setInterpolateMethod(int $method): bool
    {
    }

    public function setOrientation(int $orientation): bool
    {
    }
}

class ImagickDraw
{
    public function affine(array $affine): bool
    {
    }

    public function annotation(float $x, float $y, string $text): bool
    {
    }

    public function arc(
        float $start_x,
        float $start_y,
        float $end_x,
        float $end_y,
        float $start_angle,
        float $end_angle,
    ): bool {
    }

    public function bezier(array $coordinates): bool
    {
    }

    public function circle(float $origin_x, float $origin_y, float $perimeter_x, float $perimeter_y): bool
    {
    }

    public function clear(): bool
    {
    }

    public function clone(): ImagickDraw
    {
    }

    public function color(float $x, float $y, int $paint): bool
    {
    }

    public function comment(string $comment): bool
    {
    }

    public function composite(int $composite, float $x, float $y, float $width, float $height, Imagick $image): bool
    {
    }

    public function destroy(): bool
    {
    }

    public function ellipse(
        float $origin_x,
        float $origin_y,
        float $radius_x,
        float $radius_y,
        float $angle_start,
        float $angle_end,
    ): bool {
    }

    public function getClipPath(): false|string
    {
    }

    public function getClipRule(): int
    {
    }

    public function getClipUnits(): int
    {
    }

    public function getFillColor(): ImagickPixel
    {
    }

    public function getFillOpacity(): float
    {
    }

    public function getFillRule(): int
    {
    }

    public function getFont(): string
    {
    }

    public function getFontFamily(): string
    {
    }

    public function getFontSize(): float
    {
    }

    public function getFontStretch(): int
    {
    }

    public function getFontStyle(): int
    {
    }

    public function getFontWeight(): int
    {
    }

    public function getGravity(): int
    {
    }

    public function getStrokeAntialias(): bool
    {
    }

    public function getStrokeColor(): ImagickPixel
    {
    }

    public function getStrokeDashArray(): array
    {
    }

    public function getStrokeDashOffset(): float
    {
    }

    public function getStrokeLineCap(): int
    {
    }

    public function getStrokeLineJoin(): int
    {
    }

    public function getStrokeMiterLimit(): int
    {
    }

    public function getStrokeOpacity(): float
    {
    }

    public function getStrokeWidth(): float
    {
    }

    public function getTextAlignment(): int
    {
    }

    public function getTextAntialias(): bool
    {
    }

    public function getTextDecoration(): int
    {
    }

    public function getTextEncoding(): false|string
    {
    }

    public function getTextInterlineSpacing(): float
    {
    }

    public function getTextInterwordSpacing(): float
    {
    }

    public function getTextKerning(): float
    {
    }

    public function getTextUnderColor(): ImagickPixel
    {
    }

    public function getVectorGraphics(): string
    {
    }

    public function line(float $start_x, float $start_y, float $end_x, float $end_y): bool
    {
    }

    public function matte(float $x, float $y, int $paint): bool
    {
    }

    public function pathClose(): bool
    {
    }

    public function pathCurveToAbsolute(float $x1, float $y1, float $x2, float $y2, float $x, float $y): bool
    {
    }

    public function pathCurveToQuadraticBezierAbsolute(float $x1, float $y1, float $x_end, float $y): bool
    {
    }

    public function pathCurveToQuadraticBezierRelative(float $x1, float $y1, float $x_end, float $y): bool
    {
    }

    public function pathCurveToQuadraticBezierSmoothAbsolute(float $x, float $y): bool
    {
    }

    public function pathCurveToQuadraticBezierSmoothRelative(float $x, float $y): bool
    {
    }

    public function pathCurveToRelative(float $x1, float $y1, float $x2, float $y2, float $x, float $y): bool
    {
    }

    public function pathCurveToSmoothAbsolute(float $x2, float $y2, float $x, float $y): bool
    {
    }

    public function pathCurveToSmoothRelative(float $x2, float $y2, float $x, float $y): bool
    {
    }

    public function pathEllipticArcAbsolute(
        float $rx,
        float $ry,
        float $x_axis_rotation,
        bool $large_arc,
        bool $sweep,
        float $x,
        float $y,
    ): bool {
    }

    public function pathEllipticArcRelative(
        float $rx,
        float $ry,
        float $x_axis_rotation,
        bool $large_arc,
        bool $sweep,
        float $x,
        float $y,
    ): bool {
    }

    public function pathFinish(): bool
    {
    }

    public function pathLineToAbsolute(float $x, float $y): bool
    {
    }

    public function pathLineToHorizontalAbsolute(float $x): bool
    {
    }

    public function pathLineToHorizontalRelative(float $x): bool
    {
    }

    public function pathLineToRelative(float $x, float $y): bool
    {
    }

    public function pathLineToVerticalAbsolute(float $y): bool
    {
    }

    public function pathLineToVerticalRelative(float $y): bool
    {
    }

    public function pathMoveToAbsolute(float $x, float $y): bool
    {
    }

    public function pathMoveToRelative(float $x, float $y): bool
    {
    }

    public function pathStart(): bool
    {
    }

    public function point(float $x, float $y): bool
    {
    }

    public function polygon(array $coordinates): bool
    {
    }

    public function polyline(array $coordinates): bool
    {
    }

    public function pop(): bool
    {
    }

    public function popClipPath(): bool
    {
    }

    public function popDefs(): bool
    {
    }

    public function popPattern(): bool
    {
    }

    public function push(): bool
    {
    }

    public function pushClipPath(string $clip_mask_id): bool
    {
    }

    public function pushDefs(): bool
    {
    }

    public function pushPattern(string $pattern_id, float $x, float $y, float $width, float $height): bool
    {
    }

    public function rectangle(float $top_left_x, float $top_left_y, float $bottom_right_x, float $bottom_right_y): bool
    {
    }

    public function render(): bool
    {
    }

    public function resetVectorGraphics(): bool
    {
    }

    public function rotate(float $degrees): bool
    {
    }

    public function roundRectangle(
        float $top_left_x,
        float $top_left_y,
        float $bottom_right_x,
        float $bottom_right_y,
        float $rounding_x,
        float $rounding_y,
    ): bool {
    }

    public function scale(float $x, float $y): bool
    {
    }

    public function setClipPath(string $clip_mask): bool
    {
    }

    public function setClipRule(int $fillrule): bool
    {
    }

    public function setClipUnits(int $pathunits): bool
    {
    }

    public function setFillAlpha(float $alpha): bool
    {
    }

    public function setFillColor(ImagickPixel|string $fill_color): bool
    {
    }

    public function setFillOpacity(float $opacity): bool
    {
    }

    public function setFillPatternURL(string $fill_url): bool
    {
    }

    public function setFillRule(int $fillrule): bool
    {
    }

    public function setFont(string $font_name): bool
    {
    }

    public function setFontFamily(string $font_family): bool
    {
    }

    public function setFontSize(float $point_size): bool
    {
    }

    public function setFontStretch(int $stretch): bool
    {
    }

    public function setFontStyle(int $style): bool
    {
    }

    public function setFontWeight(int $weight): bool
    {
    }

    public function setGravity(int $gravity): bool
    {
    }

    public function setResolution(float $resolution_x, float $resolution_y): bool
    {
    }

    public function setStrokeAlpha(float $alpha): bool
    {
    }

    public function setStrokeAntialias(bool $enabled): bool
    {
    }

    public function setStrokeColor(ImagickPixel|string $color): bool
    {
    }

    public function setStrokeDashArray(null|array $dashes): bool
    {
    }

    public function setStrokeDashOffset(float $dash_offset): bool
    {
    }

    public function setStrokeLineCap(int $linecap): bool
    {
    }

    public function setStrokeLineJoin(int $linejoin): bool
    {
    }

    public function setStrokeMiterLimit(int $miterlimit): bool
    {
    }

    public function setStrokeOpacity(float $opacity): bool
    {
    }

    public function setStrokePatternURL(string $stroke_url): bool
    {
    }

    public function setStrokeWidth(float $width): bool
    {
    }

    public function setTextAlignment(int $align): bool
    {
    }

    public function setTextAntialias(bool $antialias): bool
    {
    }

    public function setTextDecoration(int $decoration): bool
    {
    }

    public function setTextEncoding(string $encoding): bool
    {
    }

    public function setTextInterlineSpacing(float $spacing): bool
    {
    }

    public function setTextInterwordSpacing(float $spacing): bool
    {
    }

    public function setTextKerning(float $kerning): bool
    {
    }

    public function setTextUnderColor(ImagickPixel|string $under_color): bool
    {
    }

    public function setVectorGraphics(string $xml): bool
    {
    }

    public function setViewbox(int $left_x, int $top_y, int $right_x, int $bottom_y): bool
    {
    }

    public function skewX(float $degrees): bool
    {
    }

    public function skewY(float $degrees): bool
    {
    }

    public function translate(float $x, float $y): bool
    {
    }
}

class ImagickException extends Exception
{
}

class ImagickDrawException extends Exception
{
}

class ImagickPixelIteratorException extends Exception
{
}

class ImagickPixelException extends Exception
{
}

class ImagickKernelException extends Exception
{
}
