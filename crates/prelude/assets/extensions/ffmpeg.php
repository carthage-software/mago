<?php

class ffmpeg_movie
{
    /**
     * @param string $path_to_media
     * @param bool $persistent
     */
    public function __construct($path_to_media, $persistent) {}

    /**
     * @return int
     */
    public function getDuration() {}

    /**
     * @return int
     */
    public function getFrameCount() {}

    /**
     * @return int
     */
    public function getFrameRate() {}

    /**
     * @return string
     */
    public function getFilename() {}

    /**
     * @return string
     */
    public function getComment() {}

    /**
     * @return string
     */
    public function getTitle() {}

    /**
     * @return string
     */
    public function getAuthor() {}

    /**
     * @return string
     */
    public function getArtist() {}

    /**
     * @return string
     */
    public function getCopyright() {}

    /**
     * @return string
     */
    public function getGenre() {}

    /**
     * @return string|int
     */
    public function getTrackNumber() {}

    /**
     * @return string|int
     */
    public function getYear() {}

    /**
     * @return int
     */
    public function getFrameHeight() {}

    /**
     * @return int
     */
    public function getFrameWidth() {}

    public function getPixelFormat() {}

    /**
     * @return int
     */
    public function getBitRate() {}

    /**
     * @return int
     */
    public function getVideoBitRate() {}

    /**
     * @return int
     */
    public function getAudioBitRate() {}

    /**
     * @return int
     */
    public function getAudioSampleRate() {}

    /**
     * @return int
     */
    public function getFrameNumber() {}

    /**
     * @return string
     */
    public function getVideoCodec() {}

    /**
     * @return string
     */
    public function getAudioCodec() {}

    /**
     * @return int
     */
    public function getAudioChannels() {}

    /**
     * @return bool
     */
    public function hasAudio() {}

    /**
     * @return bool
     */
    public function hasVideo() {}

    /**
     * @param int $framenumber
     *
     * @return ffmpeg_frame
     */
    public function getFrame($framenumber) {}

    /**
     * @return ffmpeg_frame
     */
    public function getNextKeyFrame() {}
}

class ffmpeg_frame
{
    /**
     * @param resource $gd_image
     */
    public function __construct($gd_image) {}

    /**
     * @return int
     */
    public function getWidth() {}

    /**
     * @return int
     */
    public function getHeight() {}

    /**
     * @return int
     */
    public function getPTS() {}

    /**
     * @return int
     */
    public function getPresentationTimestamp() {}

    /**
     * @param int $width
     * @param int $height
     * @param int $crop_top
     * @param int $crop_bottom
     * @param int $crop_left
     * @param int $crop_right
     */
    public function resize($width, $height, $crop_top = 0, $crop_bottom = 0, $crop_left = 0, $crop_right = 0) {}

    /**
     * @param int $crop_top
     * @param int $crop_bottom
     * @param int $crop_left
     * @param int $crop_right
     */
    public function crop($crop_top, $crop_bottom = 0, $crop_left = 0, $crop_right = 0) {}

    /**
     * @return resource
     */
    public function toGDImage() {}
}

class ffmpeg_animated_gif
{
    /**
     * @param string $output_file_path
     * @param int $width
     * @param int $height
     * @param int $frame_rate
     * @param int $loop_count
     */
    public function __construct($output_file_path, $width, $height, $frame_rate, $loop_count = 0) {}

    public function addFrame(ffmpeg_frame $frame_to_add) {}
}
