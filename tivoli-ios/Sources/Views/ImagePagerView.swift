import SwiftUI

struct ImagePagerView: View {
    @Binding var images: [ImageSummary]
    let initialIndex: Int

    @Environment(APIClient.self) private var api
    @Environment(\.dismiss) private var dismiss
    @State private var currentIndex: Int
    @State private var showTagEditor = false
    @State private var isZoomed = false
    @State private var dragOffset: CGFloat = 0

    init(images: Binding<[ImageSummary]>, initialIndex: Int) {
        self._images = images
        self.initialIndex = initialIndex
        self._currentIndex = State(initialValue: initialIndex)
    }

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            TabView(selection: $currentIndex) {
                ForEach(Array(images.enumerated()), id: \.element.id) { index, image in
                    ZoomableImage(url: api.imageURL(uuid: image.uuid), isZoomed: $isZoomed)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .tag(index)
                        .onTapGesture {
                            withAnimation { showTagEditor.toggle() }
                        }
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .automatic))
            .ignoresSafeArea()
            .offset(y: dragOffset)
            .gesture(
                DragGesture(minimumDistance: 40, coordinateSpace: .global)
                    .onChanged { value in
                        guard !isZoomed else { return }
                        if value.translation.height > 0 {
                            dragOffset = value.translation.height
                        }
                    }
                    .onEnded { value in
                        guard !isZoomed else { return }
                        let velocity = value.predictedEndLocation.y - value.location.y
                        if value.translation.height > 100 || velocity > 500 {
                            dismiss()
                        } else {
                            withAnimation(.spring(duration: 0.3)) {
                                dragOffset = 0
                            }
                        }
                    },
                isEnabled: !isZoomed
            )

            if showTagEditor, currentIndex < images.count {
                VStack {
                    Spacer()
                    TagEditorView(imageUUID: images[currentIndex].uuid)
                        .transition(.move(edge: .bottom))
                }
            }
        }
        .background(Color.black.opacity(1 - Double(dragOffset) / 400).ignoresSafeArea())
        .statusBarHidden()
    }
}
