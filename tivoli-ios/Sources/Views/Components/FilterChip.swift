import SwiftUI

struct FilterChip: View {
    let label: String
    let isSelected: Bool
    var isDisabled: Bool = false
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(label)
                .font(.caption)
                .padding(.horizontal, 10)
                .padding(.vertical, 5)
                .background(
                    isSelected
                        ? Color.accentColor
                        : isDisabled ? Color(.systemGray5) : Color(.systemGray6)
                )
                .foregroundColor(
                    isSelected ? .white : isDisabled ? .gray : .primary
                )
                .clipShape(.capsule)
        }
        .buttonStyle(.plain)
        .disabled(isDisabled)
        .animation(.snappy(duration: 0.2), value: isSelected)
    }
}
