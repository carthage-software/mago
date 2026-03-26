<?php

class SWFShape
{
    public function __construct() {}

    public function setLine() {}

    public function addFill() {}

    public function setLeftFill() {}

    public function setRightFill() {}

    public function movePenTo() {}

    public function movePen() {}

    public function drawLineTo() {}

    public function drawLine() {}

    public function drawCurveTo() {}

    public function drawCurve() {}

    public function drawGlyph() {}

    public function drawCircle() {}

    public function drawArc() {}

    public function drawCubic() {}

    public function drawCubicTo() {}
}

class SWFFill
{
    public function __construct() {}

    public function moveTo() {}

    public function scaleTo() {}

    public function rotateTo() {}

    public function skewXTo() {}

    public function skewYTo() {}
}

class SWFGradient
{
    public function __construct() {}

    public function addEntry() {}
}

class SWFBitmap
{
    public function __construct() {}

    public function getWidth() {}

    public function getHeight() {}
}

class SWFText
{
    public function __construct() {}

    public function setFont() {}

    public function setHeight() {}

    public function setSpacing() {}

    public function setColor() {}

    public function moveTo() {}

    public function addString() {}

    public function addUTF8String() {}

    public function getWidth() {}

    public function getUTF8Width() {}

    public function getAscent() {}

    public function getDescent() {}

    public function getLeading() {}
}

class SWFTextField
{
    public function __construct() {}

    public function setFont() {}

    public function setBounds() {}

    public function align() {}

    public function setHeight() {}

    public function setLeftMargin() {}

    public function setRightMargin() {}

    public function setMargins() {}

    public function setIndentation() {}

    public function setLineSpacing() {}

    public function setColor() {}

    public function setName() {}

    public function addString() {}

    public function setPadding() {}

    public function addChars() {}
}

class SWFFont
{
    public function __construct() {}

    public function getWidth() {}

    public function getUTF8Width() {}

    public function getAscent() {}

    public function getDescent() {}

    public function getLeading() {}

    public function getShape() {}
}

class SWFDisplayItem
{
    public function moveTo() {}

    public function move() {}

    public function scaleTo() {}

    public function scale() {}

    public function rotateTo() {}

    public function rotate() {}

    public function skewXTo() {}

    public function skewX() {}

    public function skewYTo() {}

    public function skewY() {}

    public function setMatrix() {}

    public function setDepth() {}

    public function setRatio() {}

    public function addColor() {}

    public function multColor() {}

    public function setName() {}

    public function addAction() {}

    public function remove() {}

    public function setMaskLevel() {}

    public function endMask() {}

    public function getX() {}

    public function getY() {}

    public function getXScale() {}

    public function getYScale() {}

    public function getXSkew() {}

    public function getYSkew() {}

    public function getRot() {}
}

class SWFMovie
{
    public function __construct() {}

    public function nextFrame() {}

    public function labelFrame() {}

    public function add() {}

    public function remove() {}

    public function output() {}

    public function save() {}

    public function saveToFile() {}

    public function setBackground() {}

    public function setRate() {}

    public function setDimension() {}

    public function setFrames() {}

    public function streamMP3() {}

    public function addExport() {}

    public function writeExports() {}

    public function startSound() {}

    public function stopSound() {}

    public function importChar() {}

    public function importFont() {}

    public function addFont() {}

    public function protect() {}

    public function namedAnchor() {}
}

class SWFButton
{
    public function __construct() {}

    public function setHit() {}

    public function setOver() {}

    public function setUp() {}

    public function setDown() {}

    public function setAction() {}

    public function addShape() {}

    public function setMenu() {}

    public function addAction() {}

    public function addSound() {}
}

class SWFAction
{
    public function __construct() {}
}

class SWFMorph
{
    public function __construct() {}

    public function getShape1() {}

    public function getShape2() {}
}

class SWFSprite
{
    public function __construct() {}

    public function add() {}

    public function remove() {}

    public function nextFrame() {}

    public function labelFrame() {}

    public function setFrames() {}

    public function startSound() {}

    public function stopSound() {}
}

class SWFSound
{
    public function __construct() {}
}

class SWFFontChar
{
    public function addChars() {}

    public function addUTF8Chars() {}
}

class SWFSoundInstance
{
    public function noMultiple() {}

    public function loopInPoint() {}

    public function loopOutPoint() {}

    public function loopCount() {}
}

class SWFVideoStream
{
    public function __construct() {}

    public function setdimension() {}

    public function getnumframes() {}
}

function ming_setcubicthreshold(int $threshold): void {}

function ming_setscale(float $scale): void {}

function ming_useswfversion(int $version): void {}

function ming_keypress(string $char): int {}

function ming_useconstants(int $use): void {}

function ming_setswfcompression(int $level): void {}

const MING_NEW = 1;

const MING_ZLIB = 1;

const SWFBUTTON_HIT = 8;

const SWFBUTTON_DOWN = 4;

const SWFBUTTON_OVER = 2;

const SWFBUTTON_UP = 1;

const SWFBUTTON_MOUSEUPOUTSIDE = 64;

const SWFBUTTON_DRAGOVER = 160;

const SWFBUTTON_DRAGOUT = 272;

const SWFBUTTON_MOUSEUP = 8;

const SWFBUTTON_MOUSEDOWN = 4;

const SWFBUTTON_MOUSEOUT = 2;

const SWFBUTTON_MOUSEOVER = 1;

const SWFFILL_RADIAL_GRADIENT = 18;

const SWFFILL_LINEAR_GRADIENT = 16;

const SWFFILL_TILED_BITMAP = 64;

const SWFFILL_CLIPPED_BITMAP = 65;

const SWFTEXTFIELD_HASLENGTH = 2;

const SWFTEXTFIELD_NOEDIT = 8;

const SWFTEXTFIELD_PASSWORD = 16;

const SWFTEXTFIELD_MULTILINE = 32;

const SWFTEXTFIELD_WORDWRAP = 64;

const SWFTEXTFIELD_DRAWBOX = 2048;

const SWFTEXTFIELD_NOSELECT = 4096;

const SWFTEXTFIELD_HTML = 512;

const SWFTEXTFIELD_USEFONT = 256;

const SWFTEXTFIELD_AUTOSIZE = 16384;

const SWFTEXTFIELD_ALIGN_LEFT = 0;

const SWFTEXTFIELD_ALIGN_RIGHT = 1;

const SWFTEXTFIELD_ALIGN_CENTER = 2;

const SWFTEXTFIELD_ALIGN_JUSTIFY = 3;

const SWFACTION_ONLOAD = 1;

const SWFACTION_ENTERFRAME = 2;

const SWFACTION_UNLOAD = 4;

const SWFACTION_MOUSEMOVE = 8;

const SWFACTION_MOUSEDOWN = 16;

const SWFACTION_MOUSEUP = 32;

const SWFACTION_KEYDOWN = 64;

const SWFACTION_KEYUP = 128;

const SWFACTION_DATA = 256;

const SWF_SOUND_NOT_COMPRESSED = 0;

const SWF_SOUND_ADPCM_COMPRESSED = 16;

const SWF_SOUND_MP3_COMPRESSED = 32;

const SWF_SOUND_NOT_COMPRESSED_LE = 48;

const SWF_SOUND_NELLY_COMPRESSED = 96;

const SWF_SOUND_5KHZ = 0;

const SWF_SOUND_11KHZ = 4;

const SWF_SOUND_22KHZ = 8;

const SWF_SOUND_44KHZ = 12;

const SWF_SOUND_8BITS = 0;

const SWF_SOUND_16BITS = 2;

const SWF_SOUND_MONO = 0;

const SWF_SOUND_STEREO = 1;
