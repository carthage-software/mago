<?php declare(strict_types=1);

// Test @inheritDoc with multiple interfaces

interface Readable
{
    /**
     * @param int $offset Position to read from
     * @return string The data read
     */
    public function read( $offset);
}

interface Writable
{
    /**
     * @param string $data Data to write
     * @param int $offset Position to write to
     * @return int Number of bytes written
     */
    public function write( $data,  $offset);
}

class FileHandler implements Readable, Writable
{
    /** @inheritDoc */
    public function read(int $offset)
    {
        // Should inherit: @param int $offset, @return string
        return 'data';
    }

    /** @inheritDoc */
    public function write( $data,  $offset)
    {
        // Should inherit: @param string $data, @param int $offset, @return int
        return strlen($data);
    }
}

$handler = new FileHandler();
$data = $handler->read(0);
$written = $handler->write('test', 10);
echo "Read data: $data, Bytes written: $written\n";