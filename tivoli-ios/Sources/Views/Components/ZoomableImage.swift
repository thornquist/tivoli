import SwiftUI
import UIKit

struct ZoomableImage: UIViewRepresentable {
    let url: URL
    @Binding var isZoomed: Bool

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UIScrollView {
        let scrollView = UIScrollView()
        scrollView.delegate = context.coordinator
        scrollView.minimumZoomScale = 1
        scrollView.maximumZoomScale = 5
        scrollView.bouncesZoom = true
        scrollView.showsVerticalScrollIndicator = false
        scrollView.showsHorizontalScrollIndicator = false
        scrollView.backgroundColor = .clear

        let imageView = UIImageView()
        imageView.contentMode = .scaleAspectFit
        imageView.clipsToBounds = true
        imageView.tag = 100
        scrollView.addSubview(imageView)

        let doubleTap = UITapGestureRecognizer(
            target: context.coordinator, action: #selector(Coordinator.handleDoubleTap(_:)))
        doubleTap.numberOfTapsRequired = 2
        scrollView.addGestureRecognizer(doubleTap)

        context.coordinator.scrollView = scrollView
        context.coordinator.loadImage(url: url)

        return scrollView
    }

    func updateUIView(_ scrollView: UIScrollView, context: Context) {
        if context.coordinator.currentURL != url {
            context.coordinator.currentURL = url
            scrollView.zoomScale = 1
            context.coordinator.loadImage(url: url)
        }
    }

    class Coordinator: NSObject, UIScrollViewDelegate {
        let parent: ZoomableImage
        weak var scrollView: UIScrollView?
        var currentURL: URL?
        private var loadTask: Task<Void, Never>?

        init(parent: ZoomableImage) {
            self.parent = parent
            self.currentURL = parent.url
        }

        func loadImage(url: URL) {
            loadTask?.cancel()
            loadTask = Task { @MainActor in
                guard let scrollView else { return }
                let imageView = scrollView.viewWithTag(100) as? UIImageView
                do {
                    let image = try await ImageCache.shared.image(for: url)
                    guard !Task.isCancelled else { return }
                    imageView?.image = image
                    layoutImageView(in: scrollView)
                } catch {
                    // Image failed to load
                }
            }
        }

        private func layoutImageView(in scrollView: UIScrollView) {
            guard let imageView = scrollView.viewWithTag(100) as? UIImageView,
                  let image = imageView.image else { return }
            let boundsSize = scrollView.bounds.size
            guard boundsSize.width > 0, boundsSize.height > 0 else { return }
            let imageSize = image.size
            let widthScale = boundsSize.width / imageSize.width
            let heightScale = boundsSize.height / imageSize.height
            let fitScale = min(widthScale, heightScale)
            let fittedSize = CGSize(
                width: imageSize.width * fitScale,
                height: imageSize.height * fitScale)
            imageView.frame = CGRect(origin: .zero, size: fittedSize)
            scrollView.contentSize = fittedSize
            centerImageView(in: scrollView)
        }

        private func centerImageView(in scrollView: UIScrollView) {
            guard let imageView = scrollView.viewWithTag(100) as? UIImageView else { return }
            let boundsSize = scrollView.bounds.size
            let contentSize = imageView.frame.size

            let xOffset = max(0, (boundsSize.width - contentSize.width * scrollView.zoomScale) / 2)
            let yOffset = max(0, (boundsSize.height - contentSize.height * scrollView.zoomScale) / 2)
            imageView.center = CGPoint(
                x: scrollView.contentSize.width / 2 + xOffset,
                y: scrollView.contentSize.height / 2 + yOffset)
        }

        // MARK: - UIScrollViewDelegate

        func viewForZooming(in scrollView: UIScrollView) -> UIView? {
            scrollView.viewWithTag(100)
        }

        func scrollViewDidZoom(_ scrollView: UIScrollView) {
            centerImageView(in: scrollView)
            DispatchQueue.main.async {
                self.parent.isZoomed = scrollView.zoomScale > 1.01
            }
        }

        func scrollViewDidEndZooming(_ scrollView: UIScrollView, with view: UIView?, atScale scale: CGFloat) {
            DispatchQueue.main.async {
                self.parent.isZoomed = scale > 1.01
            }
        }

        // MARK: - Double Tap

        @objc func handleDoubleTap(_ gesture: UITapGestureRecognizer) {
            guard let scrollView else { return }
            if scrollView.zoomScale > 1.01 {
                scrollView.setZoomScale(1, animated: true)
            } else {
                let location = gesture.location(in: scrollView.viewWithTag(100))
                let zoomRect = CGRect(
                    x: location.x - 50, y: location.y - 50,
                    width: 100, height: 100)
                scrollView.zoom(to: zoomRect, animated: true)
            }
        }
    }
}
