import SwiftUI

struct ZoomableImage: View {
    let url: URL
    @State private var scale: CGFloat = 1
    @State private var lastScale: CGFloat = 1
    @State private var offset: CGSize = .zero
    @State private var lastOffset: CGSize = .zero

    var body: some View {
        CachedAsyncImage(url: url, contentMode: .fit)
            .scaleEffect(scale)
            .offset(offset)
            .gesture(magnifyGesture)
            .gesture(panGesture, isEnabled: scale > 1)
            .onTapGesture(count: 2) {
                withAnimation(.snappy(duration: 0.25)) {
                    if scale > 1 {
                        scale = 1
                        lastScale = 1
                        offset = .zero
                        lastOffset = .zero
                    } else {
                        scale = 3
                        lastScale = 3
                    }
                }
            }
            .onChange(of: url) { _, _ in
                scale = 1
                lastScale = 1
                offset = .zero
                lastOffset = .zero
            }
    }

    private var magnifyGesture: some Gesture {
        MagnifyGesture()
            .onChanged { value in
                scale = lastScale * value.magnification
            }
            .onEnded { _ in
                if scale < 1 {
                    withAnimation(.snappy(duration: 0.25)) { scale = 1 }
                    lastScale = 1
                } else {
                    lastScale = scale
                }
            }
    }

    private var panGesture: some Gesture {
        DragGesture()
            .onChanged { value in
                offset = CGSize(
                    width: lastOffset.width + value.translation.width,
                    height: lastOffset.height + value.translation.height
                )
            }
            .onEnded { _ in
                lastOffset = offset
            }
    }
}
