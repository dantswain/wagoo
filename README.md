# Wagoo

My [rust](https://www.rust-lang.org/) [wgpu](https://wgpu.rs/) playground.

Here is an example screenshot with 1000 simulated Lorenz attractor trajectories:

![Lorenz Attractor](https://github.com/dantswain/wagoo/raw/main/screenshots/lorenz1.jpg)

This is largely based on [Learn WGPU](https://sotrh.github.io/learn-wgpu/) as a starting point, specifically the [final tutorial source code](https://github.com/sotrh/learn-wgpu/tree/master/code/intermediate/tutorial13-threading/).

Ostensibly my goal here is to genereate some desktop wallpaper images.  The code is not really optimized for FPS but rather flexibility/iterability and visual tweaks (if it could be said to be optimized at all).

This code is pretty rough!

* I am a rust novice and using this project to learn.
* I've used OpenGL with C++ before, but it's been about 10 years and the translation from OpenGL to wgpu/wgsl is not trivial. 
* My approach with this code is to tweak the animation by tweaking the source code.  Therefore things are not really organized super well for configuration or parameterization.
* There is also a lot of commented-out and/or dead code that I wanted to keep around for reference or because I use it in some simulations.

## Simulation

The simulation is generally a particle (drawn as a sphere) moving through 3D space with its trajectory drawn as a line.  Changing the dynamical model of the particles movement changes the simulation.

Different models are implemented in [src/dynamics.rs](src/dynamics.rs):

* `Circler` - Moves roughly in a circle in the XY plane while asymptotically approaching `z = 0`, with some random perturbation.  This was my initial simple model for building the system.

* `Lorenz` - Implements a [Lorenz Attractor](https://en.wikipedia.org/wiki/Lorenz_system).

## Graphics / GPU Techniques

One of the reasons I'm making this a public repo is because I'm hoping maybe it will help others who are similarly struggling to figure out how to translate ideas from OpenGL to wgpu/wgsl.  Here is a list of techniques I've used.  If you have trouble finding them in the source code, feel free to open an issue and ask.

* Generate and draw an instanced sphere.
* Draw lines.
* Pass data into the shader using uniform and vertex buffers.
* Multi-pass rendering.
* Capturing the renderer output to a texture buffer.
* Post-processing by drawing to a full-frame texture quad (this would be a component step in producing a [bloom effect](https://en.wikipedia.org/wiki/Bloom_(shader_effect)), but haven't yet wired up the whole thing).
* Saving screenshots

## Build & Run

Assuming you have rust 1.52.1 or greater, this should be as simple as:

```
cargo run
```

* Control the camera with WASD (translation) and mouse click-drag (pitch & yaw)
* Pause the simulation/animation with space bar
* Capture a screenshot with the enter key
* Exit with the escape key (sometimes you have to also hit Ctrl-C)

## License

[MIT](LICENSE.txt)
