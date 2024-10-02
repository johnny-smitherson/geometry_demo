set -ex
rm -f screenshot/*.png
cargo run --bin take_screenshots_and_exit
cd screenshot
ffmpeg  -y -i '%05d.png' -filter_complex "fps=50,scale=1024:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=32[p];[s1][p]paletteuse=dither=bayer" output.gif

ffmpeg  -y -i '%05d.png' -filter_complex "fps=50,scale=1024:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=32[p];[s1][p]paletteuse=dither=bayer" -c:v libx264 -profile:v baseline -level 3.0 -pix_fmt yuv420p  output.mp4
cp screenshot0.png tmp
rm -f *.png
mv tmp output.png
ls -alh output.gif