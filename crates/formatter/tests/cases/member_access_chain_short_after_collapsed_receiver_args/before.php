<?php

new UploadFileToSignedUrlRequest(
    upload_url: $upload->url,
    content_type: $type,
)->send($client)->dtoOrFail();
