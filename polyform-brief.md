# Polyform — Project Brief

## What it is

Polyform is a **lattice synthesizer**: an instrument for performing geometric and mathematical transformations on a regular substrate, in real time. The substrate is a 2D grid. The performance act is composing operations that warp, fold, and lift that grid into images.

It belongs to the lineage of video synthesis — Rutt-Etra, LZX Industries, Sleepy Circuits, Cat Full of Ghosts, Andrei Jay, Signal Culture — and shares that tradition's defining gesture: **images are constructed from non-image input through a structured chain of artist-composed transformations**, not captured from the world. Where audio synthesizers shape signals into sound and traditional video synthesizers shape signals into images, Polyform shapes a coordinate substrate into images.

What distinguishes Polyform is the substrate and the vocabulary. Most video synths process raster signals or oscillators — quantities that vary over time. Polyform processes a *manifold* — a 2D grid that exists all at once and is transformed as a geometric object. The operations are named in mathematical terms (z^n, möbius, conjugate, conformal map, displacement field) rather than electrical terms (filter, mixer, integrator). The artist composes maps; the instrument renders the result.

This grounds the practice in mathematical structure that is both visible and performable, and that distinction is the heart of the project.

## Why it exists

Higher-dimensional mathematics is one of the most beautiful things humans have ever discovered, and one of the least available. The barrier to entry is almost entirely linguistic and notational — the underlying *intuitions* are physical and accessible if you can find a way to feel them. Polyform exists to be one such way.

The instrument is built around a small set of convictions:

**Mathematics has a felt dimension that text cannot reach.** A Möbius transformation drawn on paper is a static diagram. A Möbius transformation modulated against tempo, watched warping a grid in real time alongside other operations, is something you *experience*. The experience accumulates into intuition without any deliberate study.

**Intuition can precede comprehension.** A player doesn't need to understand the mathematics to play Polyform, in the same way a player doesn't need to understand FM synthesis to play a DX7. Unlike a synth, though, the underlying object being manipulated *is* the mathematics itself — exposed, named, and available. The path from playing to understanding is open whenever the player is ready to walk it.

**Higher-dimensional thinking is a teachable skill.** It is not an inborn gift reserved for mathematicians. The reason most people never develop it is that they never see it operating on something they can hold in their attention. Polyform's dimension readout — the live count of structural and temporal degrees of freedom in the active configuration — is part of how that gets taught. Watching the count rise and fall as you build up a patch makes "ten dimensions" something you've felt rather than read.

**Visibility is the precondition for play.** The 2D grid is the anchor not because it's a limitation but because it's what makes everything else legible. The viewer always sees both what changed and what was there before. Strip the grid and you have an image; keep the grid and you have a demonstration. This is also what makes the tool contemplative — you can rest your attention on what's happening rather than chasing it.

The project's animating ambition is that someone who plays Polyform for an hour leaves with something they didn't have when they started: a felt sense of what mathematical structure is, how it can be composed, and what it means for a space to have many dimensions. That sense is the actual product. The images are a side effect.

## Identity constraint

The 2D grid is the anchor. Polyform may grow to include 3D lattices, particle emitters seeded from grid intersections, attractors traced through the gridded coordinate space, wave interference visualized as grid displacement, and other extensions — but only insofar as those extensions honor the substrate. Anything that strays into untethered fluid simulation, raw oscillator art, or generative work without a regular reference becomes a different instrument.

The constraint isn't aesthetic restriction; it's what makes the mathematics legible and the instrument coherent. Without it, Polyform would dissolve into a general-purpose math toy. With it, the project has an identity that scales as it grows.

## Architecture

Polyform's data model is a stack of typed operations applied to a base 2D lattice, followed by a stack of 3D lift operations, viewed through a configurable camera. Every operation is parameterized; every parameter can be set, ranged, animated, and modulated against a global tempo with rational or irrational ratios.

The pipeline has three stages:

**Operations** — ordered composition of ℝ² → ℝ² maps applied to the grid. Includes linear (translate, scale, rotate, shear, matrix), complex (z^n, 1/z, möbius, exp, log, sin, cos), polar (spiral, pinch, swirl, angle warp), wave, and non-holomorphic (conjugate, reflect, |z|) families. Composition is non-commutative; reordering changes the result.

**Lift** — sequential ℝ³ displacement applied to the post-operation lattice. Includes height-style (magnitude, signed Jacobian, real/imag part, argument), wave (sine, gaussian), and full-3D displacement (spherical projection, conical, helical, ripple) families.

**Camera** — position (orbit, distance, target, roll), lens (focal length, sensor size, anamorphic, lens shift), distortion (radial Brown-Conrady k1/k2), and quaternion tumble. The camera is not a passive viewer; it's a performable instrument layer of its own.

Every layer has a per-layer **blend** parameter. Toggling a layer animates blend between 0 and 1 over a configurable blend time, so structural changes are smooth rather than stepped. The same primitive handles preset transitions, layer reordering, and live morphing.

The **dimension readout** displays the live count of structural and temporal degrees of freedom in the active configuration. This is the philosophical core of the interface made visible. It is both a diagnostic and a feature, and players are expected to use it as performance information, not just developer information.

## Preset system

A preset is a JSON document containing the complete patch state: pipeline structure, parameter values and ranges, camera, sampling, palette, tempo, display toggles. Presets carry stable layer IDs so morphing engines can match layers across presets by identity rather than position. Tempo lives in the preset so crossfades between presets can smoothly accelerate or decelerate the entire animation system as a single coherent gesture.

The schema is versioned with graceful forward compatibility: unknown fields are ignored, missing fields fall back to defaults, type mismatches warn rather than fail. This is the portable asset of the project — the renderer is replaceable; the preset format isn't.

## Trajectory

The project is being developed in three phases:

**Phase 1 — Web prototype (current).** A React + three.js component serves as the design tool and validation surface. Iteration is fast; ideas are tested quickly. This is where the instrument's identity is being discovered through use.

**Phase 2 — Native Rust application.** A standalone Windows app built with wgpu and egui, structured as a control window plus an output window. Spout integration is deferred: the SpoutCapture utility (from the Spout author) bridges the output window into Resolume and other Spout receivers as a separate process, removing integration risk. The same preset format moves between web and native.

**Phase 3 — Native C++ rewrite.** Once the design has stabilized through performance use and feedback worth shipping for, a C++ rewrite with direct Spout SDK integration produces the polished, distributable version. The preset format is the bridge that makes this rewrite a translation rather than a redesign. This phase is genuinely speculative and contingent on what the earlier phases reveal.

## Use modes

Two distinct modes, both first-class:

**Live performance / VJ instrument.** Animation on, parameters modulated against a global tempo with polyrhythmic ratios, presets morphing into each other, output sent to Resolume via Spout. The dimension readout and operation chain become part of the visual identity of the performance — the visible math is itself part of what's being performed.

**Static structure design.** Animation off, precise control over each parameter, presets dialed in by hand. Still images and looped sequences for print, design work, and contemplative pieces.

A third mode is implicit and worth naming: **exploratory play.** A user with no performance goal at all, sitting with the instrument the way one might sit with a kaleidoscope or a Newton's cradle. Polyform is intentionally rewarding in this mode. The contemplative quality of the visible grid — always legible, always ready to be rested on — is what makes that work.

## What this is not

Polyform is not a general-purpose math visualization tool, not a VJ workstation, not a node-based shader editor, not a video mixer. It is a single-purpose instrument with a focused identity: lattice synthesis, with a 2D regular substrate as anchor, in the lineage of video synthesis, intended to be played — and through being played, to make higher-dimensional mathematical thinking something a person can feel rather than something they have to study.
