fn main() -> anyhow::Result<()> {
    pollster::block_on(raytracer::run())
}
