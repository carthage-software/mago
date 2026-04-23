<?php

class Store
{
    public function delete(Key $key): void
    {
        $write = new BulkWrite();
        $write->delete([
            '_id' => (string) $key,
            'token' => $this->getUniqueToken($key),
        ], ['limit' => 1]);

        $this->getManager()->executeBulkWrite($this->namespace, $write, [
            'writeConcern' => new WriteConcern(WriteConcern::MAJORITY),
        ]);
    }
}
