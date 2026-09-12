<?php

class Demo
{
    public function run($targetDir, $idCustomer, $idDummyarticle)
    {
        if (true) {
            if (true) {
                foreach ($_FILES as $Fileid) {
                    $targetFile = date('Y-m-d', time()) . '_' . $idCustomer . '_' . $idDummyarticle . '_' . basename(
                        $Fileid['name']
                    );
                    $targetPath = $targetDir . $targetFile;
                }
            }
        }
    }
}
