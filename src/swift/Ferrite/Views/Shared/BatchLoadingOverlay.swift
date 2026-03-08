import SwiftUI

struct BatchLoadingOverlay: View {
    let state: DecompilerService.BatchLoadingState

    var body: some View {
        ZStack {
            Color(red: 25/255, green: 25/255, blue: 28/255)
                .ignoresSafeArea()

            MagneticBreathView()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Magnetic Breath

private struct MagneticBreathView: View {
    private static let pulseCount = 3
    private static let coreBaseRadius: CGFloat = 24
    private static let coreAmplitude: CGFloat = 4
    private static let breatheSpeed: Double = 0.8
    private static let morphSpeed: Double = 0.3
    private static let maxPulseRadius: CGFloat = 90
    private static let pulseCycleDuration: Double = 3.0

    var body: some View {
        TimelineView(.animation) { timeline in
            let t = timeline.date.timeIntervalSinceReferenceDate

            Canvas { context, size in
                let center = CGPoint(x: size.width / 2, y: size.height / 2)

                // Core breathing radius
                let breathe = sin(t * Self.breatheSpeed * 2 * .pi)
                let coreRadius = Self.coreBaseRadius + Self.coreAmplitude * breathe

                // Shape morph factor: 0 = circle, 1 = hexagon
                let morphFactor = 0.5 + 0.5 * sin(t * Self.morphSpeed * 2 * .pi)

                // --- Pulse rings ---
                for i in 0..<Self.pulseCount {
                    let phaseOffset = Double(i) / Double(Self.pulseCount)
                    let cycleT = (t / Self.pulseCycleDuration + phaseOffset).truncatingRemainder(dividingBy: 1.0)
                    let progress = cycleT // 0→1

                    // Ease out for smooth deceleration
                    let easedProgress = 1.0 - pow(1.0 - progress, 2.0)

                    let pulseRadius = coreRadius + (Self.maxPulseRadius - coreRadius) * easedProgress
                    let pulseOpacity = 0.35 * pow(1.0 - progress, 1.8)
                    let pulseLineWidth = 1.4 * (1.0 - progress * 0.8)

                    // Pulse rings also morph, but less so as they expand
                    let pulseMorph = morphFactor * (1.0 - easedProgress)
                    let pulsePath = morphedShape(
                        center: center,
                        radius: pulseRadius,
                        morphFactor: pulseMorph,
                        segments: 128
                    )

                    context.stroke(
                        pulsePath,
                        with: .color(.white.opacity(pulseOpacity)),
                        lineWidth: pulseLineWidth
                    )
                }

                // --- Center glow ---
                let glowRadius: CGFloat = coreRadius * 2.5
                let glowRect = CGRect(
                    x: center.x - glowRadius,
                    y: center.y - glowRadius,
                    width: glowRadius * 2,
                    height: glowRadius * 2
                )
                let glowPulse = 0.06 + 0.03 * breathe
                context.fill(
                    Path(ellipseIn: glowRect),
                    with: .radialGradient(
                        Gradient(colors: [
                            .white.opacity(glowPulse),
                            .clear,
                        ]),
                        center: center,
                        startRadius: 0,
                        endRadius: glowRadius
                    )
                )

                // --- Core shape ---
                let corePath = morphedShape(
                    center: center,
                    radius: coreRadius,
                    morphFactor: morphFactor,
                    segments: 128
                )

                context.stroke(
                    corePath,
                    with: .color(.white.opacity(0.7)),
                    lineWidth: 1.5
                )

                // Inner fill — very subtle
                context.fill(
                    corePath,
                    with: .color(.white.opacity(0.03))
                )
            }
            .frame(width: 200, height: 200)
        }
    }

    /// Creates a path that smoothly interpolates between a circle and a hexagon.
    /// `morphFactor`: 0 = circle, 1 = hexagon.
    private func morphedShape(
        center: CGPoint,
        radius: CGFloat,
        morphFactor: Double,
        segments: Int
    ) -> Path {
        Path { path in
            for i in 0...segments {
                let angle = (Double(i) / Double(segments)) * 2 * .pi

                // Hexagonal perturbation: distance modulation with 6-fold symmetry
                // cos(3θ)² gives a smooth hexagonal envelope
                let hex = pow(cos(3 * angle), 2)
                // Hexagon has corners further out, edges closer in
                // The perturbation shrinks the radius at edge midpoints
                let perturbation = 1.0 - 0.12 * hex
                let r = radius * (1.0 + morphFactor * (perturbation - 1.0))

                let x = center.x + r * cos(angle)
                let y = center.y + r * sin(angle)

                if i == 0 {
                    path.move(to: CGPoint(x: x, y: y))
                } else {
                    path.addLine(to: CGPoint(x: x, y: y))
                }
            }
            path.closeSubpath()
        }
    }
}
