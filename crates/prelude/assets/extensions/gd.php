<?php

/** @var int */
const IMG_AVIF = 256;

/** @var int */
const IMG_GIF = 1;

/** @var int */
const IMG_JPG = 2;

/** @var int */
const IMG_JPEG = 2;

/** @var int */
const IMG_PNG = 4;

/** @var int */
const IMG_WBMP = 8;

/** @var int */
const IMG_XPM = 16;

/** @var int */
const IMG_WEBP = 32;

/** @var int */
const IMG_BMP = 64;

/** @var int */
const IMG_TGA = 128;

/** @var int */
const IMG_WEBP_LOSSLESS = 101;

/** @var int */
const IMG_COLOR_TILED = -5;

/** @var int */
const IMG_COLOR_STYLED = -2;

/** @var int */
const IMG_COLOR_BRUSHED = -3;

/** @var int */
const IMG_COLOR_STYLEDBRUSHED = -4;

/** @var int */
const IMG_COLOR_TRANSPARENT = -6;

/** @var int */
const IMG_ARC_ROUNDED = 0;

/** @var int */
const IMG_ARC_PIE = 0;

/** @var int */
const IMG_ARC_CHORD = 1;

/** @var int */
const IMG_ARC_NOFILL = 2;

/** @var int */
const IMG_ARC_EDGED = 4;

/** @var int */
const IMG_GD2_RAW = 1;

/** @var int */
const IMG_GD2_COMPRESSED = 2;

/** @var int */
const IMG_FLIP_HORIZONTAL = 1;

/** @var int */
const IMG_FLIP_VERTICAL = 2;

/** @var int */
const IMG_FLIP_BOTH = 3;

/** @var int */
const IMG_EFFECT_REPLACE = 0;

/** @var int */
const IMG_EFFECT_ALPHABLEND = 1;

/** @var int */
const IMG_EFFECT_NORMAL = 2;

/** @var int */
const IMG_EFFECT_OVERLAY = 3;

/** @var int */
const IMG_EFFECT_MULTIPLY = 4;

/** @var int */
const IMG_CROP_DEFAULT = 0;

/** @var int */
const IMG_CROP_TRANSPARENT = 1;

/** @var int */
const IMG_CROP_BLACK = 2;

/** @var int */
const IMG_CROP_WHITE = 3;

/** @var int */
const IMG_CROP_SIDES = 4;

/** @var int */
const IMG_CROP_THRESHOLD = 5;

/** @var int */
const IMG_BELL = 1;

/** @var int */
const IMG_BESSEL = 2;

/** @var int */
const IMG_BILINEAR_FIXED = 3;

/** @var int */
const IMG_BICUBIC = 4;

/** @var int */
const IMG_BICUBIC_FIXED = 5;

/** @var int */
const IMG_BLACKMAN = 6;

/** @var int */
const IMG_BOX = 7;

/** @var int */
const IMG_BSPLINE = 8;

/** @var int */
const IMG_CATMULLROM = 9;

/** @var int */
const IMG_GAUSSIAN = 10;

/** @var int */
const IMG_GENERALIZED_CUBIC = 11;

/** @var int */
const IMG_HERMITE = 12;

/** @var int */
const IMG_HAMMING = 13;

/** @var int */
const IMG_HANNING = 14;

/** @var int */
const IMG_MITCHELL = 15;

/** @var int */
const IMG_POWER = 17;

/** @var int */
const IMG_QUADRATIC = 18;

/** @var int */
const IMG_SINC = 19;

/** @var int */
const IMG_NEAREST_NEIGHBOUR = 16;

/** @var int */
const IMG_WEIGHTED4 = 21;

/** @var int */
const IMG_TRIANGLE = 20;

/** @var int */
const IMG_AFFINE_TRANSLATE = 0;

/** @var int */
const IMG_AFFINE_SCALE = 1;

/** @var int */
const IMG_AFFINE_ROTATE = 2;

/** @var int */
const IMG_AFFINE_SHEAR_HORIZONTAL = 3;

/** @var int */
const IMG_AFFINE_SHEAR_VERTICAL = 4;

/** @var int */
const GD_BUNDLED = 0;

/** @var int */
const IMG_FILTER_NEGATE = 0;

/** @var int */
const IMG_FILTER_GRAYSCALE = 1;

/** @var int */
const IMG_FILTER_BRIGHTNESS = 2;

/** @var int */
const IMG_FILTER_CONTRAST = 3;

/** @var int */
const IMG_FILTER_COLORIZE = 4;

/** @var int */
const IMG_FILTER_EDGEDETECT = 5;

/** @var int */
const IMG_FILTER_GAUSSIAN_BLUR = 7;

/** @var int */
const IMG_FILTER_SELECTIVE_BLUR = 8;

/** @var int */
const IMG_FILTER_EMBOSS = 6;

/** @var int */
const IMG_FILTER_MEAN_REMOVAL = 9;

/** @var int */
const IMG_FILTER_SMOOTH = 10;

/** @var int */
const IMG_FILTER_PIXELATE = 11;

/** @var int */
const IMG_FILTER_SCATTER = 12;

/** @var string */
const GD_VERSION = '2.3.3';

/** @var int */
const GD_MAJOR_VERSION = 2;

/** @var int */
const GD_MINOR_VERSION = 3;

/** @var int */
const GD_RELEASE_VERSION = 3;

/** @var string */
const GD_EXTRA_VERSION = '';

/** @var int */
const PNG_NO_FILTER = 0;

/** @var int */
const PNG_FILTER_NONE = 8;

/** @var int */
const PNG_FILTER_SUB = 16;

/** @var int */
const PNG_FILTER_UP = 32;

/** @var int */
const PNG_FILTER_AVG = 64;

/** @var int */
const PNG_FILTER_PAETH = 128;

/** @var int */
const PNG_ALL_FILTERS = 248;

final class GdImage
{
}

final class GdFont
{
}

function gd_info(): array
{
}

function imageloadfont(string $filename): GdFont|false
{
}

function imagesetstyle(GdImage $image, array $style): bool
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatetruecolor(int $width, int $height): GdImage|false
{
}

function imageistruecolor(GdImage $image): bool
{
}

function imagetruecolortopalette(GdImage $image, bool $dither, int $num_colors): bool
{
}

function imagepalettetotruecolor(GdImage $image): bool
{
}

function imagecolormatch(GdImage $image1, GdImage $image2): bool
{
}

function imagesetthickness(GdImage $image, int $thickness): bool
{
}

function imagefilledellipse(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $color): bool
{
}

function imagefilledarc(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $start_angle, int $end_angle, int $color, int $style): bool
{
}

function imagealphablending(GdImage $image, bool $enable): bool
{
}

function imagesavealpha(GdImage $image, bool $enable): bool
{
}

function imagelayereffect(GdImage $image, int $effect): bool
{
}

function imagecolorallocatealpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int|false
{
}

function imagecolorresolvealpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int
{
}

function imagecolorclosestalpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int
{
}

function imagecolorexactalpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int
{
}

function imagecopyresampled(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $dst_width, int $dst_height, int $src_width, int $src_height): bool
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagerotate(GdImage $image, float $angle, int $background_color): GdImage|false
{
}

function imagesettile(GdImage $image, GdImage $tile): bool
{
}

function imagesetbrush(GdImage $image, GdImage $brush): bool
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreate(int $width, int $height): GdImage|false
{
}

function imagetypes(): int
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromstring(string $data): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromavif(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromgif(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromjpeg(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefrompng(string $filename): GdImage|false
{
}

/**
 * @param string $filename
 * @return GdImage|false
 * @psalm-ignore-falsable-return
 */
function imagecreatefromwebp(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromxbm(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromwbmp(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromgd(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromgd2(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromgd2part(string $filename, int $x, int $y, int $width, int $height): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefrombmp(string $filename): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecreatefromtga(string $filename): GdImage|false
{
}

function imagexbm(GdImage $image, string|null $filename, int|null $foreground_color = null): bool
{
}

function imageavif(GdImage $image, $file = null, int $quality = -1, int $speed = -1): bool
{
}

function imagegif(GdImage $image, $file = null): bool
{
}

function imagepng(GdImage $image, $file = null, int $quality = -1, int $filters = -1): bool
{
}

function imagewebp(GdImage $image, $file = null, int $quality = -1): bool
{
}

function imagejpeg(GdImage $image, $file = null, int $quality = -1): bool
{
}

function imagewbmp(GdImage $image, $file = null, int|null $foreground_color = null): bool
{
}

function imagegd(GdImage $image, string|null $file = null): bool
{
}

function imagegd2(GdImage $image, string|null $file = null, int $chunk_size = 128, int $mode = 1): bool
{
}

function imagebmp(GdImage $image, $file = null, bool $compressed = true): bool
{
}

function imagedestroy(GdImage $image): bool
{
}

function imagecolorallocate(GdImage $image, int $red, int $green, int $blue): int|false
{
}

function imagepalettecopy(GdImage $dst, GdImage $src): void
{
}

function imagecolorat(GdImage $image, int $x, int $y): int|false
{
}

function imagecolorclosest(GdImage $image, int $red, int $green, int $blue): int
{
}

function imagecolorclosesthwb(GdImage $image, int $red, int $green, int $blue): int
{
}

function imagecolordeallocate(GdImage $image, int $color): bool
{
}

function imagecolorresolve(GdImage $image, int $red, int $green, int $blue): int
{
}

function imagecolorexact(GdImage $image, int $red, int $green, int $blue): int
{
}

function imagecolorset(GdImage $image, int $color, int $red, int $green, int $blue, int $alpha = 0): false|null
{
}

function imagecolorsforindex(GdImage $image, int $color): array
{
}

function imagegammacorrect(GdImage $image, float $input_gamma, float $output_gamma): bool
{
}

function imagesetpixel(GdImage $image, int $x, int $y, int $color): bool
{
}

function imageline(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
}

function imagedashedline(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
}

function imagerectangle(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
}

function imagefilledrectangle(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
}

function imagearc(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $start_angle, int $end_angle, int $color): bool
{
}

function imageellipse(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $color): bool
{
}

function imagefilltoborder(GdImage $image, int $x, int $y, int $border_color, int $color): bool
{
}

function imagefill(GdImage $image, int $x, int $y, int $color): bool
{
}

function imagecolorstotal(GdImage $image): int
{
}

function imagecolortransparent(GdImage $image, int|null $color = null): int
{
}

function imageinterlace(GdImage $image, bool|null $enable = null): bool
{
}

function imagepolygon(GdImage $image, array $points, int $num_points_or_color, int|null $color = null): bool
{
}

function imageopenpolygon(GdImage $image, array $points, int $num_points_or_color, int|null $color = null): bool
{
}

function imagefilledpolygon(GdImage $image, array $points, int $num_points_or_color, int|null $color = null): bool
{
}

function imagefontwidth(GdFont|int $font): int
{
}

function imagefontheight(GdFont|int $font): int
{
}

function imagechar(GdImage $image, GdFont|int $font, int $x, int $y, string $char, int $color): bool
{
}

function imagecharup(GdImage $image, GdFont|int $font, int $x, int $y, string $char, int $color): bool
{
}

function imagestring(GdImage $image, GdFont|int $font, int $x, int $y, string $string, int $color): bool
{
}

function imagestringup(GdImage $image, GdFont|int $font, int $x, int $y, string $string, int $color): bool
{
}

function imagecopy(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $src_width, int $src_height): bool
{
}

function imagecopymerge(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $src_width, int $src_height, int $pct): bool
{
}

function imagecopymergegray(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $src_width, int $src_height, int $pct): bool
{
}

function imagecopyresized(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $dst_width, int $dst_height, int $src_width, int $src_height): bool
{
}

function imagesx(GdImage $image): int
{
}

function imagesy(GdImage $image): int
{
}

function imagesetclip(GdImage $image, int $x1, int $y1, int $x2, int $y2): bool
{
}

function imagegetclip(GdImage $image): array
{
}

function imageftbbox(float $size, float $angle, string $font_filename, string $string, array $options = []): array|false
{
}

function imagefttext(GdImage $image, float $size, float $angle, int $x, int $y, int $color, string $font_filename, string $text, array $options = []): array|false
{
}

function imagettfbbox(float $size, float $angle, string $font_filename, string $string, array $options = []): array|false
{
}

function imagettftext(GdImage $image, float $size, float $angle, int $x, int $y, int $color, string $font_filename, string $text, array $options = []): array|false
{
}

function imagefilter(GdImage $image, int $filter, ...$args): bool
{
}

function imageconvolution(GdImage $image, array $matrix, float $divisor, float $offset): bool
{
}

function imageflip(GdImage $image, int $mode): bool
{
}

function imageantialias(GdImage $image, bool $enable): bool
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecrop(GdImage $image, array $rectangle): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagecropauto(GdImage $image, int $mode = 0, float $threshold = 0.5, int $color = -1): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imagescale(GdImage $image, int $width, int $height = -1, int $mode = 3): GdImage|false
{
}

/**
 * @psalm-ignore-falsable-return
 */
function imageaffine(GdImage $image, array $affine, array|null $clip = null): GdImage|false
{
}

function imageaffinematrixget(int $type, $options): array|false
{
}

function imageaffinematrixconcat(array $matrix1, array $matrix2): array|false
{
}

function imagegetinterpolation(GdImage $image): int
{
}

function imagesetinterpolation(GdImage $image, int $method = 3): bool
{
}

function imageresolution(GdImage $image, int|null $resolution_x = null, int|null $resolution_y = null): array|bool
{
}

