import SwiftUI

struct ConnectView: View {
    let onConnect: (APIClient) -> Void

    @State private var urlText = ""
    @State private var isConnecting = false
    @State private var errorMessage: String?
    @State private var serverStore = ServerStore()

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            // App branding
            VStack(spacing: 8) {
                Text("Tivoli")
                    .font(.system(size: 48, weight: .bold, design: .serif))
                Text("Portrait Gallery")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .tracking(1.5)
                    .textCase(.uppercase)
            }
            .padding(.bottom, 48)

            // Connection form
            VStack(spacing: 16) {
                TextField("Server address", text: $urlText, prompt: Text("http://192.168.1.5:3000"))
                    .textContentType(.URL)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .padding(14)
                    .background(Color(.systemGray6), in: .rect(cornerRadius: 12))
                    .font(.body.monospaced())

                Button {
                    Task { await connect(urlString: urlText) }
                } label: {
                    Group {
                        if isConnecting {
                            ProgressView()
                                .tint(.white)
                        } else {
                            Text("Connect")
                                .font(.headline)
                        }
                    }
                    .frame(maxWidth: .infinity)
                    .frame(height: 50)
                    .background(
                        urlText.isEmpty || isConnecting
                            ? Color.accentColor.opacity(0.4)
                            : Color.accentColor,
                        in: .rect(cornerRadius: 12)
                    )
                    .foregroundStyle(.white)
                }
                .disabled(isConnecting || urlText.isEmpty)

                if let error = errorMessage {
                    Text(error)
                        .font(.caption)
                        .foregroundStyle(.red)
                        .multilineTextAlignment(.center)
                        .transition(.opacity)
                }
            }
            .padding(.horizontal, 32)

            Spacer()

            // Recent servers
            if !serverStore.servers.isEmpty {
                VStack(spacing: 0) {
                    Text("RECENT")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.tertiary)
                        .tracking(1)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 32)
                        .padding(.bottom, 12)

                    VStack(spacing: 0) {
                        ForEach(serverStore.servers) { server in
                            Button {
                                Task { await connect(urlString: server.url) }
                            } label: {
                                HStack {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(server.label)
                                            .font(.subheadline.weight(.medium))
                                        Text(server.url)
                                            .font(.caption.monospaced())
                                            .foregroundStyle(.secondary)
                                    }
                                    Spacer()
                                    Image(systemName: "arrow.right.circle")
                                        .foregroundStyle(.tertiary)
                                }
                                .padding(.horizontal, 32)
                                .padding(.vertical, 12)
                                .contentShape(Rectangle())
                            }
                            .buttonStyle(.plain)

                            if server.id != serverStore.servers.last?.id {
                                Divider().padding(.leading, 32)
                            }
                        }
                    }
                }
                .padding(.bottom, 24)
            }
        }
        .animation(.snappy, value: errorMessage != nil)
    }

    private func connect(urlString: String) async {
        let raw = urlString.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: raw), url.scheme != nil, url.host != nil else {
            errorMessage = "Enter a valid URL (e.g. http://192.168.1.5:3000)"
            return
        }
        isConnecting = true
        errorMessage = nil
        let client = APIClient(baseURL: url)
        do {
            try await client.testConnection()
            serverStore.addOrUpdate(url: raw)
            onConnect(client)
        } catch {
            errorMessage = error.localizedDescription
        }
        isConnecting = false
    }
}
