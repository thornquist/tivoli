import SwiftUI

@main
struct TivoliApp: App {
    @State private var apiClient: APIClient?
    @State private var useThumbnails = true
    @State private var prefetchCount = 50

    var body: some Scene {
        WindowGroup {
            Group {
                if let apiClient {
                    MainView(useThumbnails: useThumbnails, prefetchCount: prefetchCount)
                        .environment(apiClient)
                } else {
                    ConnectView { client, thumbnails, prefetch in
                        useThumbnails = thumbnails
                        prefetchCount = prefetch
                        apiClient = client
                    }
                }
            }
            .statusBarHidden()
            .persistentSystemOverlays(.hidden)
        }
    }
}
