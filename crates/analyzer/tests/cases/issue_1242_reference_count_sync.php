<?php

final class Repro
{
    public function run(): void
    {
        $riff = [
            'WAVE' => ['rgad' => [['data' => 'ABCDEFGH']]],
            'rgad' => ['track' => ['name' => '', 'originator' => '']],
        ];
        $riffRaw = [
            'rgad' => ['track' => ['name' => 0, 'originator' => 0], 'radio' => 0],
        ];

        $wave = &$riff['WAVE'];

        if (isset($wave['rgad'][0]['data'])) {
            $rgadData = &$wave['rgad'][0]['data'];
            $rgad = &$riffRaw['rgad'];
            $track = &$rgad['track'];

            $rgad['radio'] = strlen(substr($rgadData, 4, 2));
            $radioBits = str_pad(decbin($rgad['radio']), 16, '0', STR_PAD_LEFT);

            $track['name'] = bindec(substr($radioBits, 0, 3));
            $track['originator'] = bindec(substr($radioBits, 3, 3));

            if (($track['name'] != 0) && ($track['originator'] != 0)) {
                $riff['rgad']['track']['name'] = 'x';
                $riff['rgad']['track']['originator'] = 'y';
            }
        }
    }
}
