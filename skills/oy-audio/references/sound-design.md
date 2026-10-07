# From Sound Intent to Synthesis Structure

Determine what the sound should communicate before choosing sources. Map descriptions such as light, heavy, crisp, soft, near, distant, rough, bright, mechanical, or organic to envelopes, spectrum, modulation, density, and space. Weight often comes from the low-frequency body and decay; hardness from a short broadband attack; distance from weaker high frequencies and a lower direct-sound proportion. Apply these relationships across interfaces, games, interactive installations, film transitions, and abstract sound design, without treating them as fixed presets.

## Layers and timing

| Layer role | Common construction | What to inspect |
| --- | --- | --- |
| Attack, contact, fracture | Short noise, narrow pulses, rapidly descending tones | Attack speed, harsh high frequencies, unintended DC steps |
| Body, weight, material | Low tones, bandpassed noise, harmonic or inharmonic resonances | Low-frequency buildup, recognizable material, connection to the attack |
| Friction, motion, flow | Noise bands, amplitude modulation, sweeps, pan movement | Direction, natural variation, exposed repetition in loops |
| Release, resonance, space | Exponential decay, sparse echoes, diffuse tails | Complete release, masking of the following event |

Choose layers for recognizability rather than complexity. Establish a convincing body first, then add necessary attack and texture. Compare versions with controlled variables; changing seed, duration, level, and timbre together makes improvements hard to assess.

## Starting points for common categories

- **Short feedback and interaction cues:** Short tones and clear envelopes often communicate more directly than elaborate spatial effects. Keep success, failure, selection, and notification sounds within one timbral family, distinguishing them through interval direction, rhythm, density, and brightness. Frequent actions should sound shorter and lighter; avoid long tails on every click. The `confirm` example demonstrates layered tones, not a requirement that all positive feedback rise in pitch.
- **Collisions and impacts:** Separate the contact transient, physical weight, and material resonance. Metal can use multiple decaying tones with noninteger frequency ratios; wood tends toward darker, shorter resonances; energy bursts can combine low tones and noise. Adjust mass, hardness, and size deliberately rather than only changing playback speed.
- **Repeated actions and foley:** Construct each action from contact, friction, and release, then arrange the event sequence. Footsteps, key presses, tool strikes, raindrops, and mechanical movement all need variation appropriate to their rhythm and material. Small changes to spectrum, intensity, and timing reduce mechanical repetition. Add background noise only when the scene calls for it; do not automatically add room tone to every isolated asset.
- **Ambience and sustained textures:** Combine a continuous bed with motion at different timescales and sparse events. Wind, steam, fluids, machinery, and fictional energy fields can share this structure, while their spectra, periodicity, and transient density must serve their distinct identities. Thunder needs unfolding low-frequency rumbles rather than a single blast. Vary pulse spacing in fire or electrical textures to avoid an accidental metronome.
- **Pass-bys, risers, and transitions:** Give energy, frequency, and speed a clear starting point and destination. Cross-modulate low and high bands to move brightness, or use pitch sweeps with correctly integrated phase. Specify alignment points when synchronizing with visuals or beats, and preserve the tail after the pass-by point.

## Quality decisions

UI cues can start around -12 to -6 dBFS peak; choose headroom for other effects according to their mix context. These are starting points rather than universal targets. Preserve transients and dynamics instead of flattening a sound to hit a number. Compare variants at fixed gain or adjust them together against one reference.

Spatial effects should downmix sensibly to mono. Keep the low-frequency body stable where appropriate; create width through intentional differences rather than simply inverting one channel. Loops must match in spectrum, motion, and energy. Setting only the first and last samples to zero can create a periodic gap.

When realism is requested and synthesis cannot convey the needed material detail, prefer layering and processing recordings supplied by the user or authorized for use. Do not present synthesized audio as a recording. When no sample is available and stylization is acceptable, produce a useful synthesized version and describe its nature accurately.
