import SwiftUI

@main
struct TivoliApp: App {
    @State private var apiClient: APIClient?

    var body: some Scene {
        WindowGroup {
            Group {
                if let apiClient {
                    MainView()
                        .environment(apiClient)
                } else {
                    ConnectView { client in
                        apiClient = client
                    }
                }
            }
            .statusBarHidden()
            .persistentSystemOverlays(.hidden)
        }
    }
}
