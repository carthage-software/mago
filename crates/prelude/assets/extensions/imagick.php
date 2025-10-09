<?php

class Imagick
{
    public const COLOR_BLACK = 11;
    public const COLOR_BLUE = 12;
    public const COLOR_CYAN = 13;
    public const COLOR_GREEN = 14;
    public const COLOR_RED = 15;
    public const COLOR_YELLOW = 16;
    public const COLOR_MAGENTA = 17;
    public const COLOR_OPACITY = 18;
    public const COLOR_ALPHA = 19;
    public const COLOR_FUZZ = 20;
    public const IMAGICK_EXTNUM = 30403;
    public const IMAGICK_EXTVER = "3.4.3";
    public const QUANTUM_RANGE = 65535;
    public const USE_ZEND_MM = 0;
    public const COMPOSITE_DEFAULT = 40;
    public const COMPOSITE_UNDEFINED = 0;
    public const COMPOSITE_NO = 1;
    public const COMPOSITE_ADD = 2;
    public const COMPOSITE_ATOP = 3;
    public const COMPOSITE_BLEND = 4;
    public const COMPOSITE_BUMPMAP = 5;
    public const COMPOSITE_CLEAR = 7;
    public const COMPOSITE_COLORBURN = 8;
    public const COMPOSITE_COLORDODGE = 9;
    public const COMPOSITE_COLORIZE = 10;
    public const COMPOSITE_COPYBLACK = 11;
    public const COMPOSITE_COPYBLUE = 12;
    public const COMPOSITE_COPY = 13;
    public const COMPOSITE_COPYCYAN = 14;
    public const COMPOSITE_COPYGREEN = 15;
    public const COMPOSITE_COPYMAGENTA = 16;
    public const COMPOSITE_COPYOPACITY = 17;
    public const COMPOSITE_COPYRED = 18;
    public const COMPOSITE_COPYYELLOW = 19;
    public const COMPOSITE_DARKEN = 20;
    public const COMPOSITE_DSTATOP = 21;
    public const COMPOSITE_DST = 22;
    public const COMPOSITE_DSTIN = 23;
    public const COMPOSITE_DSTOUT = 24;
    public const COMPOSITE_DSTOVER = 25;
    public const COMPOSITE_DIFFERENCE = 26;
    public const COMPOSITE_DISPLACE = 27;
    public const COMPOSITE_DISSOLVE = 28;
    public const COMPOSITE_EXCLUSION = 29;
    public const COMPOSITE_HARDLIGHT = 30;
    public const COMPOSITE_HUE = 31;
    public const COMPOSITE_IN = 32;
    public const COMPOSITE_LIGHTEN = 33;
    public const COMPOSITE_LUMINIZE = 35;
    public const COMPOSITE_MINUS = 36;
    public const COMPOSITE_MODULATE = 37;
    public const COMPOSITE_MULTIPLY = 38;
    public const COMPOSITE_OUT = 39;
    public const COMPOSITE_OVER = 40;
    public const COMPOSITE_OVERLAY = 41;
    public const COMPOSITE_PLUS = 42;
    public const COMPOSITE_REPLACE = 43;
    public const COMPOSITE_SATURATE = 44;
    public const COMPOSITE_SCREEN = 45;
    public const COMPOSITE_SOFTLIGHT = 46;
    public const COMPOSITE_SRCATOP = 47;
    public const COMPOSITE_SRC = 48;
    public const COMPOSITE_SRCIN = 49;
    public const COMPOSITE_SRCOUT = 50;
    public const COMPOSITE_SRCOVER = 51;
    public const COMPOSITE_SUBTRACT = 52;
    public const COMPOSITE_THRESHOLD = 53;
    public const COMPOSITE_XOR = 54;
    public const COMPOSITE_CHANGEMASK = 6;
    public const COMPOSITE_LINEARLIGHT = 34;
    public const COMPOSITE_DIVIDE = 55;
    public const COMPOSITE_DISTORT = 56;
    public const COMPOSITE_BLUR = 57;
    public const COMPOSITE_PEGTOPLIGHT = 58;
    public const COMPOSITE_VIVIDLIGHT = 59;
    public const COMPOSITE_PINLIGHT = 60;
    public const COMPOSITE_LINEARDODGE = 61;
    public const COMPOSITE_LINEARBURN = 62;
    public const COMPOSITE_MATHEMATICS = 63;
    public const COMPOSITE_MODULUSADD = 2;
    public const COMPOSITE_MODULUSSUBTRACT = 52;
    public const COMPOSITE_MINUSDST = 36;
    public const COMPOSITE_DIVIDEDST = 55;
    public const COMPOSITE_DIVIDESRC = 64;
    public const COMPOSITE_MINUSSRC = 65;
    public const COMPOSITE_DARKENINTENSITY = 66;
    public const COMPOSITE_LIGHTENINTENSITY = 67;
    public const MONTAGEMODE_FRAME = 1;
    public const MONTAGEMODE_UNFRAME = 2;
    public const MONTAGEMODE_CONCATENATE = 3;
    public const STYLE_NORMAL = 1;
    public const STYLE_ITALIC = 2;
    public const STYLE_OBLIQUE = 3;
    public const STYLE_ANY = 4;
    public const FILTER_UNDEFINED = 0;
    public const FILTER_POINT = 1;
    public const FILTER_BOX = 2;
    public const FILTER_TRIANGLE = 3;
    public const FILTER_HERMITE = 4;
    public const FILTER_HANNING = 5;
    public const FILTER_HAMMING = 6;
    public const FILTER_BLACKMAN = 7;
    public const FILTER_GAUSSIAN = 8;
    public const FILTER_QUADRATIC = 9;
    public const FILTER_CUBIC = 10;
    public const FILTER_CATROM = 11;
    public const FILTER_MITCHELL = 12;
    public const FILTER_LANCZOS = 22;
    public const FILTER_BESSEL = 13;
    public const FILTER_SINC = 14;
    public const FILTER_KAISER = 16;
    public const FILTER_WELSH = 17;
    public const FILTER_PARZEN = 18;
    public const FILTER_LAGRANGE = 21;
    public const FILTER_SENTINEL = 31;
    public const FILTER_BOHMAN = 19;
    public const FILTER_BARTLETT = 20;
    public const FILTER_JINC = 13;
    public const FILTER_SINCFAST = 15;
    public const FILTER_ROBIDOUX = 26;
    public const FILTER_LANCZOSSHARP = 23;
    public const FILTER_LANCZOS2 = 24;
    public const FILTER_LANCZOS2SHARP = 25;
    public const FILTER_ROBIDOUXSHARP = 27;
    public const FILTER_COSINE = 28;
    public const FILTER_SPLINE = 29;
    public const FILTER_LANCZOSRADIUS = 30;
    public const IMGTYPE_UNDEFINED = 0;
    public const IMGTYPE_BILEVEL = 1;
    public const IMGTYPE_GRAYSCALE = 2;
    public const IMGTYPE_GRAYSCALEMATTE = 3;
    public const IMGTYPE_PALETTE = 4;
    public const IMGTYPE_PALETTEMATTE = 5;
    public const IMGTYPE_TRUECOLOR = 6;
    public const IMGTYPE_TRUECOLORMATTE = 7;
    public const IMGTYPE_COLORSEPARATION = 8;
    public const IMGTYPE_COLORSEPARATIONMATTE = 9;
    public const IMGTYPE_OPTIMIZE = 10;
    public const IMGTYPE_PALETTEBILEVELMATTE = 11;
    public const RESOLUTION_UNDEFINED = 0;
    public const RESOLUTION_PIXELSPERINCH = 1;
    public const RESOLUTION_PIXELSPERCENTIMETER = 2;
    public const COMPRESSION_UNDEFINED = 0;
    public const COMPRESSION_NO = 1;
    public const COMPRESSION_BZIP = 2;
    public const COMPRESSION_FAX = 6;
    public const COMPRESSION_GROUP4 = 7;
    public const COMPRESSION_JPEG = 8;
    public const COMPRESSION_JPEG2000 = 9;
    public const COMPRESSION_LOSSLESSJPEG = 10;
    public const COMPRESSION_LZW = 11;
    public const COMPRESSION_RLE = 12;
    public const COMPRESSION_ZIP = 13;
    public const COMPRESSION_DXT1 = 3;
    public const COMPRESSION_DXT3 = 4;
    public const COMPRESSION_DXT5 = 5;
    public const COMPRESSION_ZIPS = 14;
    public const COMPRESSION_PIZ = 15;
    public const COMPRESSION_PXR24 = 16;
    public const COMPRESSION_B44 = 17;
    public const COMPRESSION_B44A = 18;
    public const COMPRESSION_LZMA = 19;
    public const COMPRESSION_JBIG1 = 20;
    public const COMPRESSION_JBIG2 = 21;
    public const PAINT_POINT = 1;
    public const PAINT_REPLACE = 2;
    public const PAINT_FLOODFILL = 3;
    public const PAINT_FILLTOBORDER = 4;
    public const PAINT_RESET = 5;
    public const GRAVITY_NORTHWEST = 1;
    public const GRAVITY_NORTH = 2;
    public const GRAVITY_NORTHEAST = 3;
    public const GRAVITY_WEST = 4;
    public const GRAVITY_CENTER = 5;
    public const GRAVITY_EAST = 6;
    public const GRAVITY_SOUTHWEST = 7;
    public const GRAVITY_SOUTH = 8;
    public const GRAVITY_SOUTHEAST = 9;
    public const GRAVITY_FORGET = 0;
    public const GRAVITY_STATIC = 10;
    public const STRETCH_NORMAL = 1;
    public const STRETCH_ULTRACONDENSED = 2;
    public const STRETCH_EXTRACONDENSED = 3;
    public const STRETCH_CONDENSED = 4;
    public const STRETCH_SEMICONDENSED = 5;
    public const STRETCH_SEMIEXPANDED = 6;
    public const STRETCH_EXPANDED = 7;
    public const STRETCH_EXTRAEXPANDED = 8;
    public const STRETCH_ULTRAEXPANDED = 9;
    public const STRETCH_ANY = 10;
    public const ALIGN_UNDEFINED = 0;
    public const ALIGN_LEFT = 1;
    public const ALIGN_CENTER = 2;
    public const ALIGN_RIGHT = 3;
    public const DECORATION_NO = 1;
    public const DECORATION_UNDERLINE = 2;
    public const DECORATION_OVERLINE = 3;
    public const DECORATION_LINETROUGH = 4;
    public const DECORATION_LINETHROUGH = 4;
    public const NOISE_UNIFORM = 1;
    public const NOISE_GAUSSIAN = 2;
    public const NOISE_MULTIPLICATIVEGAUSSIAN = 3;
    public const NOISE_IMPULSE = 4;
    public const NOISE_LAPLACIAN = 5;
    public const NOISE_POISSON = 6;
    public const NOISE_RANDOM = 7;
    public const CHANNEL_UNDEFINED = 0;
    public const CHANNEL_RED = 1;
    public const CHANNEL_GRAY = 1;
    public const CHANNEL_CYAN = 1;
    public const CHANNEL_GREEN = 2;
    public const CHANNEL_MAGENTA = 2;
    public const CHANNEL_BLUE = 4;
    public const CHANNEL_YELLOW = 4;
    public const CHANNEL_ALPHA = 8;
    public const CHANNEL_OPACITY = 8;
    public const CHANNEL_MATTE = 8;
    public const CHANNEL_BLACK = 32;
    public const CHANNEL_INDEX = 32;
    public const CHANNEL_ALL = 134217727;
    public const CHANNEL_DEFAULT = 134217719;
    public const CHANNEL_RGBA = 15;
    public const CHANNEL_TRUEALPHA = 64;
    public const CHANNEL_RGBS = 128;
    public const CHANNEL_GRAY_CHANNELS = 128;
    public const CHANNEL_SYNC = 256;
    public const CHANNEL_COMPOSITES = 47;
    public const METRIC_UNDEFINED = 0;
    public const METRIC_ABSOLUTEERRORMETRIC = 1;
    public const METRIC_MEANABSOLUTEERROR = 2;
    public const METRIC_MEANERRORPERPIXELMETRIC = 3;
    public const METRIC_MEANSQUAREERROR = 4;
    public const METRIC_PEAKABSOLUTEERROR = 5;
    public const METRIC_PEAKSIGNALTONOISERATIO = 6;
    public const METRIC_ROOTMEANSQUAREDERROR = 7;
    public const METRIC_NORMALIZEDCROSSCORRELATIONERRORMETRIC = 8;
    public const METRIC_FUZZERROR = 9;
    public const PIXEL_CHAR = 1;
    public const PIXEL_DOUBLE = 2;
    public const PIXEL_FLOAT = 3;
    public const PIXEL_INTEGER = 4;
    public const PIXEL_LONG = 5;
    public const PIXEL_QUANTUM = 6;
    public const PIXEL_SHORT = 7;
    public const EVALUATE_UNDEFINED = 0;
    public const EVALUATE_ADD = 1;
    public const EVALUATE_AND = 2;
    public const EVALUATE_DIVIDE = 3;
    public const EVALUATE_LEFTSHIFT = 4;
    public const EVALUATE_MAX = 5;
    public const EVALUATE_MIN = 6;
    public const EVALUATE_MULTIPLY = 7;
    public const EVALUATE_OR = 8;
    public const EVALUATE_RIGHTSHIFT = 9;
    public const EVALUATE_SET = 10;
    public const EVALUATE_SUBTRACT = 11;
    public const EVALUATE_XOR = 12;
    public const EVALUATE_POW = 13;
    public const EVALUATE_LOG = 14;
    public const EVALUATE_THRESHOLD = 15;
    public const EVALUATE_THRESHOLDBLACK = 16;
    public const EVALUATE_THRESHOLDWHITE = 17;
    public const EVALUATE_GAUSSIANNOISE = 18;
    public const EVALUATE_IMPULSENOISE = 19;
    public const EVALUATE_LAPLACIANNOISE = 20;
    public const EVALUATE_MULTIPLICATIVENOISE = 21;
    public const EVALUATE_POISSONNOISE = 22;
    public const EVALUATE_UNIFORMNOISE = 23;
    public const EVALUATE_COSINE = 24;
    public const EVALUATE_SINE = 25;
    public const EVALUATE_ADDMODULUS = 26;
    public const EVALUATE_MEAN = 27;
    public const EVALUATE_ABS = 28;
    public const EVALUATE_EXPONENTIAL = 29;
    public const EVALUATE_MEDIAN = 30;
    public const EVALUATE_SUM = 31;
    public const COLORSPACE_UNDEFINED = 0;
    public const COLORSPACE_RGB = 1;
    public const COLORSPACE_GRAY = 2;
    public const COLORSPACE_TRANSPARENT = 3;
    public const COLORSPACE_OHTA = 4;
    public const COLORSPACE_LAB = 5;
    public const COLORSPACE_XYZ = 6;
    public const COLORSPACE_YCBCR = 7;
    public const COLORSPACE_YCC = 8;
    public const COLORSPACE_YIQ = 9;
    public const COLORSPACE_YPBPR = 10;
    public const COLORSPACE_YUV = 11;
    public const COLORSPACE_CMYK = 12;
    public const COLORSPACE_SRGB = 13;
    public const COLORSPACE_HSB = 14;
    public const COLORSPACE_HSL = 15;
    public const COLORSPACE_HWB = 16;
    public const COLORSPACE_REC601LUMA = 17;
    public const COLORSPACE_REC709LUMA = 19;
    public const COLORSPACE_LOG = 21;
    public const COLORSPACE_CMY = 22;
    public const COLORSPACE_LUV = 23;
    public const COLORSPACE_HCL = 24;
    public const COLORSPACE_LCH = 25;
    public const COLORSPACE_LMS = 26;
    public const COLORSPACE_LCHAB = 27;
    public const COLORSPACE_LCHUV = 28;
    public const COLORSPACE_SCRGB = 29;
    public const COLORSPACE_HSI = 30;
    public const COLORSPACE_HSV = 31;
    public const COLORSPACE_HCLP = 32;
    public const COLORSPACE_YDBDR = 33;
    public const COLORSPACE_REC601YCBCR = 18;
    public const COLORSPACE_REC709YCBCR = 20;
    public const VIRTUALPIXELMETHOD_UNDEFINED = 0;
    public const VIRTUALPIXELMETHOD_BACKGROUND = 1;
    public const VIRTUALPIXELMETHOD_CONSTANT = 2;
    public const VIRTUALPIXELMETHOD_EDGE = 4;
    public const VIRTUALPIXELMETHOD_MIRROR = 5;
    public const VIRTUALPIXELMETHOD_TILE = 7;
    public const VIRTUALPIXELMETHOD_TRANSPARENT = 8;
    public const VIRTUALPIXELMETHOD_MASK = 9;
    public const VIRTUALPIXELMETHOD_BLACK = 10;
    public const VIRTUALPIXELMETHOD_GRAY = 11;
    public const VIRTUALPIXELMETHOD_WHITE = 12;
    public const VIRTUALPIXELMETHOD_HORIZONTALTILE = 13;
    public const VIRTUALPIXELMETHOD_VERTICALTILE = 14;
    public const VIRTUALPIXELMETHOD_HORIZONTALTILEEDGE = 15;
    public const VIRTUALPIXELMETHOD_VERTICALTILEEDGE = 16;
    public const VIRTUALPIXELMETHOD_CHECKERTILE = 17;
    public const PREVIEW_UNDEFINED = 0;
    public const PREVIEW_ROTATE = 1;
    public const PREVIEW_SHEAR = 2;
    public const PREVIEW_ROLL = 3;
    public const PREVIEW_HUE = 4;
    public const PREVIEW_SATURATION = 5;
    public const PREVIEW_BRIGHTNESS = 6;
    public const PREVIEW_GAMMA = 7;
    public const PREVIEW_SPIFF = 8;
    public const PREVIEW_DULL = 9;
    public const PREVIEW_GRAYSCALE = 10;
    public const PREVIEW_QUANTIZE = 11;
    public const PREVIEW_DESPECKLE = 12;
    public const PREVIEW_REDUCENOISE = 13;
    public const PREVIEW_ADDNOISE = 14;
    public const PREVIEW_SHARPEN = 15;
    public const PREVIEW_BLUR = 16;
    public const PREVIEW_THRESHOLD = 17;
    public const PREVIEW_EDGEDETECT = 18;
    public const PREVIEW_SPREAD = 19;
    public const PREVIEW_SOLARIZE = 20;
    public const PREVIEW_SHADE = 21;
    public const PREVIEW_RAISE = 22;
    public const PREVIEW_SEGMENT = 23;
    public const PREVIEW_SWIRL = 24;
    public const PREVIEW_IMPLODE = 25;
    public const PREVIEW_WAVE = 26;
    public const PREVIEW_OILPAINT = 27;
    public const PREVIEW_CHARCOALDRAWING = 28;
    public const PREVIEW_JPEG = 29;
    public const RENDERINGINTENT_UNDEFINED = 0;
    public const RENDERINGINTENT_SATURATION = 1;
    public const RENDERINGINTENT_PERCEPTUAL = 2;
    public const RENDERINGINTENT_ABSOLUTE = 3;
    public const RENDERINGINTENT_RELATIVE = 4;
    public const INTERLACE_UNDEFINED = 0;
    public const INTERLACE_NO = 1;
    public const INTERLACE_LINE = 2;
    public const INTERLACE_PLANE = 3;
    public const INTERLACE_PARTITION = 4;
    public const INTERLACE_GIF = 5;
    public const INTERLACE_JPEG = 6;
    public const INTERLACE_PNG = 7;
    public const FILLRULE_UNDEFINED = 0;
    public const FILLRULE_EVENODD = 1;
    public const FILLRULE_NONZERO = 2;
    public const PATHUNITS_UNDEFINED = 0;
    public const PATHUNITS_USERSPACE = 1;
    public const PATHUNITS_USERSPACEONUSE = 2;
    public const PATHUNITS_OBJECTBOUNDINGBOX = 3;
    public const LINECAP_UNDEFINED = 0;
    public const LINECAP_BUTT = 1;
    public const LINECAP_ROUND = 2;
    public const LINECAP_SQUARE = 3;
    public const LINEJOIN_UNDEFINED = 0;
    public const LINEJOIN_MITER = 1;
    public const LINEJOIN_ROUND = 2;
    public const LINEJOIN_BEVEL = 3;
    public const RESOURCETYPE_UNDEFINED = 0;
    public const RESOURCETYPE_AREA = 1;
    public const RESOURCETYPE_DISK = 2;
    public const RESOURCETYPE_FILE = 3;
    public const RESOURCETYPE_MAP = 4;
    public const RESOURCETYPE_MEMORY = 5;
    public const RESOURCETYPE_TIME = 7;
    public const RESOURCETYPE_THROTTLE = 8;
    public const RESOURCETYPE_THREAD = 6;
    public const DISPOSE_UNRECOGNIZED = 0;
    public const DISPOSE_UNDEFINED = 0;
    public const DISPOSE_NONE = 1;
    public const DISPOSE_BACKGROUND = 2;
    public const DISPOSE_PREVIOUS = 3;
    public const INTERPOLATE_UNDEFINED = 0;
    public const INTERPOLATE_AVERAGE = 1;
    public const INTERPOLATE_BICUBIC = 2;
    public const INTERPOLATE_BILINEAR = 3;
    public const INTERPOLATE_FILTER = 4;
    public const INTERPOLATE_INTEGER = 5;
    public const INTERPOLATE_MESH = 6;
    public const INTERPOLATE_NEARESTNEIGHBOR = 7;
    public const INTERPOLATE_SPLINE = 8;
    public const LAYERMETHOD_UNDEFINED = 0;
    public const LAYERMETHOD_COALESCE = 1;
    public const LAYERMETHOD_COMPAREANY = 2;
    public const LAYERMETHOD_COMPARECLEAR = 3;
    public const LAYERMETHOD_COMPAREOVERLAY = 4;
    public const LAYERMETHOD_DISPOSE = 5;
    public const LAYERMETHOD_OPTIMIZE = 6;
    public const LAYERMETHOD_OPTIMIZEPLUS = 8;
    public const LAYERMETHOD_OPTIMIZETRANS = 9;
    public const LAYERMETHOD_COMPOSITE = 12;
    public const LAYERMETHOD_OPTIMIZEIMAGE = 7;
    public const LAYERMETHOD_REMOVEDUPS = 10;
    public const LAYERMETHOD_REMOVEZERO = 11;
    public const LAYERMETHOD_TRIMBOUNDS = 16;
    public const ORIENTATION_UNDEFINED = 0;
    public const ORIENTATION_TOPLEFT = 1;
    public const ORIENTATION_TOPRIGHT = 2;
    public const ORIENTATION_BOTTOMRIGHT = 3;
    public const ORIENTATION_BOTTOMLEFT = 4;
    public const ORIENTATION_LEFTTOP = 5;
    public const ORIENTATION_RIGHTTOP = 6;
    public const ORIENTATION_RIGHTBOTTOM = 7;
    public const ORIENTATION_LEFTBOTTOM = 8;
    public const DISTORTION_UNDEFINED = 0;
    public const DISTORTION_AFFINE = 1;
    public const DISTORTION_AFFINEPROJECTION = 2;
    public const DISTORTION_ARC = 9;
    public const DISTORTION_BILINEAR = 6;
    public const DISTORTION_PERSPECTIVE = 4;
    public const DISTORTION_PERSPECTIVEPROJECTION = 5;
    public const DISTORTION_SCALEROTATETRANSLATE = 3;
    public const DISTORTION_POLYNOMIAL = 8;
    public const DISTORTION_POLAR = 10;
    public const DISTORTION_DEPOLAR = 11;
    public const DISTORTION_BARREL = 14;
    public const DISTORTION_SHEPARDS = 16;
    public const DISTORTION_SENTINEL = 18;
    public const DISTORTION_BARRELINVERSE = 15;
    public const DISTORTION_BILINEARFORWARD = 6;
    public const DISTORTION_BILINEARREVERSE = 7;
    public const DISTORTION_RESIZE = 17;
    public const DISTORTION_CYLINDER2PLANE = 12;
    public const DISTORTION_PLANE2CYLINDER = 13;
    public const LAYERMETHOD_MERGE = 13;
    public const LAYERMETHOD_FLATTEN = 14;
    public const LAYERMETHOD_MOSAIC = 15;
    public const ALPHACHANNEL_ACTIVATE = 1;
    public const ALPHACHANNEL_RESET = 7;
    public const ALPHACHANNEL_SET = 8;
    public const ALPHACHANNEL_UNDEFINED = 0;
    public const ALPHACHANNEL_COPY = 3;
    public const ALPHACHANNEL_DEACTIVATE = 4;
    public const ALPHACHANNEL_EXTRACT = 5;
    public const ALPHACHANNEL_OPAQUE = 6;
    public const ALPHACHANNEL_SHAPE = 9;
    public const ALPHACHANNEL_TRANSPARENT = 10;
    public const SPARSECOLORMETHOD_UNDEFINED = 0;
    public const SPARSECOLORMETHOD_BARYCENTRIC = 1;
    public const SPARSECOLORMETHOD_BILINEAR = 7;
    public const SPARSECOLORMETHOD_POLYNOMIAL = 8;
    public const SPARSECOLORMETHOD_SPEPARDS = 16;
    public const SPARSECOLORMETHOD_VORONOI = 18;
    public const SPARSECOLORMETHOD_INVERSE = 19;
    public const DITHERMETHOD_UNDEFINED = 0;
    public const DITHERMETHOD_NO = 1;
    public const DITHERMETHOD_RIEMERSMA = 2;
    public const DITHERMETHOD_FLOYDSTEINBERG = 3;
    public const FUNCTION_UNDEFINED = 0;
    public const FUNCTION_POLYNOMIAL = 1;
    public const FUNCTION_SINUSOID = 2;
    public const ALPHACHANNEL_BACKGROUND = 2;
    public const FUNCTION_ARCSIN = 3;
    public const FUNCTION_ARCTAN = 4;
    public const ALPHACHANNEL_FLATTEN = 11;
    public const ALPHACHANNEL_REMOVE = 12;
    public const STATISTIC_GRADIENT = 1;
    public const STATISTIC_MAXIMUM = 2;
    public const STATISTIC_MEAN = 3;
    public const STATISTIC_MEDIAN = 4;
    public const STATISTIC_MINIMUM = 5;
    public const STATISTIC_MODE = 6;
    public const STATISTIC_NONPEAK = 7;
    public const STATISTIC_STANDARD_DEVIATION = 8;
    public const MORPHOLOGY_CONVOLVE = 1;
    public const MORPHOLOGY_CORRELATE = 2;
    public const MORPHOLOGY_ERODE = 3;
    public const MORPHOLOGY_DILATE = 4;
    public const MORPHOLOGY_ERODE_INTENSITY = 5;
    public const MORPHOLOGY_DILATE_INTENSITY = 6;
    public const MORPHOLOGY_DISTANCE = 7;
    public const MORPHOLOGY_OPEN = 8;
    public const MORPHOLOGY_CLOSE = 9;
    public const MORPHOLOGY_OPEN_INTENSITY = 10;
    public const MORPHOLOGY_CLOSE_INTENSITY = 11;
    public const MORPHOLOGY_SMOOTH = 12;
    public const MORPHOLOGY_EDGE_IN = 13;
    public const MORPHOLOGY_EDGE_OUT = 14;
    public const MORPHOLOGY_EDGE = 15;
    public const MORPHOLOGY_TOP_HAT = 16;
    public const MORPHOLOGY_BOTTOM_HAT = 17;
    public const MORPHOLOGY_HIT_AND_MISS = 18;
    public const MORPHOLOGY_THINNING = 19;
    public const MORPHOLOGY_THICKEN = 20;
    public const MORPHOLOGY_VORONOI = 21;
    public const MORPHOLOGY_ITERATIVE = 22;
    public const KERNEL_UNITY = 1;
    public const KERNEL_GAUSSIAN = 2;
    public const KERNEL_DIFFERENCE_OF_GAUSSIANS = 3;
    public const KERNEL_LAPLACIAN_OF_GAUSSIANS = 4;
    public const KERNEL_BLUR = 5;
    public const KERNEL_COMET = 6;
    public const KERNEL_LAPLACIAN = 7;
    public const KERNEL_SOBEL = 8;
    public const KERNEL_FREI_CHEN = 9;
    public const KERNEL_ROBERTS = 10;
    public const KERNEL_PREWITT = 11;
    public const KERNEL_COMPASS = 12;
    public const KERNEL_KIRSCH = 13;
    public const KERNEL_DIAMOND = 14;
    public const KERNEL_SQUARE = 15;
    public const KERNEL_RECTANGLE = 16;
    public const KERNEL_OCTAGON = 17;
    public const KERNEL_DISK = 18;
    public const KERNEL_PLUS = 19;
    public const KERNEL_CROSS = 20;
    public const KERNEL_RING = 21;
    public const KERNEL_PEAKS = 22;
    public const KERNEL_EDGES = 23;
    public const KERNEL_CORNERS = 24;
    public const KERNEL_DIAGONALS = 25;
    public const KERNEL_LINE_ENDS = 26;
    public const KERNEL_LINE_JUNCTIONS = 27;
    public const KERNEL_RIDGES = 28;
    public const KERNEL_CONVEX_HULL = 29;
    public const KERNEL_THIN_SE = 30;
    public const KERNEL_SKELETON = 31;
    public const KERNEL_CHEBYSHEV = 32;
    public const KERNEL_MANHATTAN = 33;
    public const KERNEL_OCTAGONAL = 34;
    public const KERNEL_EUCLIDEAN = 35;
    public const KERNEL_USER_DEFINED = 36;
    public const KERNEL_BINOMIAL = 37;
    public const DIRECTION_LEFT_TO_RIGHT = 2;
    public const DIRECTION_RIGHT_TO_LEFT = 1;
    public const NORMALIZE_KERNEL_NONE = 0;
    public const NORMALIZE_KERNEL_VALUE = 8192;
    public const NORMALIZE_KERNEL_CORRELATE = 65536;
    public const NORMALIZE_KERNEL_PERCENT = 4096;

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
