// The plan
// TODO: Be able to add a render pass onto my code
// TODO: Turn an image to grayscale
// TODO: Add some dithering


fn main() -> anyhow::Result<()> {
    pollster::block_on(oklab::run())
}
