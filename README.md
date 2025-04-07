pdfcon is a simple tool for creating pdfs from a series of images and extracting images from existing PDF files.

This was created primarily to replace imagemagick for image->pdf conversion as it was slow and memory hungry. pack can create a PDF from 700 files in 2 seconds if file optimization is turned off. If turned on, packed fill attempt to
compress png files and will re-encode jpg files with mozjpeg to try to reduce their file size before adding them to the PDF.

Unpack is a work in progress. At the moment it takes significantly longer to unpack a PDF than it takes to pack one. This is due to lopd needing to build the entire document object before we can start extracting image streams.
Building the document takes around four minutes for large PDF's. This is far too long and we don't need to do this so I'm in the process of writing a custom parser to find and extract image streams without modeling the entire
doc in memory.
