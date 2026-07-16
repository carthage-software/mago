<?php

new UploadFileToSignedUrlRequest(
    upload_url: $upload->upload_url,
    content_type: $file->getMimeType() ?? 'application/octet-stream',
    content: (string) $file->getContent(),
    upload_headers: $upload->headers,
)->send()->dtoOrFail();
