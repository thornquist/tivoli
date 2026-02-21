import SwiftUI

struct TagChipView: View {
    let tag: TagRef

    var body: some View {
        Text(tag.name.replacing("-", with: " "))
            .font(.caption)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(.fill.tertiary, in: .capsule)
    }
}
