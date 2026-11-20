FILE = "./image.ppm"
if [ -f "$FILE" ]; then
  rm "$FILE"
fi

cargo run >> image.ppm
feh image.ppm
