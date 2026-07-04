import SwiftUI

struct GeneratedView: View {
    var body: some View {
        ZStack {
            Color(red: 0.961, green: 0.961, blue: 0.961)
                .ignoresSafeArea()

            VStack {
                VStack(spacing: 0) {
                    Text("Hello World")
                        .font(.largeTitle)
                        .bold()
                        .foregroundColor(Color(red: 0.2, green: 0.2, blue: 0.2))
                        .padding(.bottom, 16)

                    Text("HTML and CSS in a single file.")
                        .font(.body)
                        .foregroundColor(Color(red: 0.4, green: 0.4, blue: 0.4))
                        .padding(.bottom, 24)

                    Text("Get Started")
                        .font(.body)
                        .fontWeight(.semibold)
                        .foregroundColor(.white)
                        .padding(.horizontal, 20)
                        .padding(.vertical, 10)
                        .background(Color(red: 0.149, green: 0.388, blue: 0.922))
                        .cornerRadius(8)
                }
                .padding(32)
                .background(Color.white)
                .cornerRadius(12)
                .shadow(color: Color.black.opacity(0.1), radius: 12, x: 0, y: 4)
                .frame(maxWidth: 400)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}
