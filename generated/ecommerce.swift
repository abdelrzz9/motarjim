import SwiftUI

struct GeneratedView: View {
    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                NavbarView()
                    .padding(.bottom, 0)

                HeroView()
                    .padding(.bottom, 0)

                ProductsView()
                    .padding(.vertical, 64)
                    .padding(.horizontal, 32)
                    .frame(maxWidth: 1200)

                FooterView()
            }
            .frame(maxWidth: .infinity)
        }
    }
}

private struct NavbarView: View {
    var body: some View {
        HStack {
            Text("ShopStore")
                .font(.title)
                .bold()
                .foregroundColor(.white)

            Spacer()

            HStack(spacing: 32) {
                Text("Home")
                    .font(.body)
                    .foregroundColor(.white)
                Text("Products")
                    .font(.body)
                    .foregroundColor(.white)
                Text("Cart")
                    .font(.body)
                    .foregroundColor(.white)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 16)
        .frame(maxWidth: 1200)
        .frame(maxWidth: .infinity)
        .background(Color(red: 0.102, green: 0.102, blue: 0.18))
    }
}

private struct HeroView: View {
    var body: some View {
        VStack(spacing: 0) {
            Text("Premium Wireless Headphones")
                .font(.largeTitle)
                .bold()
                .foregroundColor(.white)
                .padding(.bottom, 16)

            Text("Experience crystal-clear audio with noise cancellation technology.")
                .font(.title3)
                .foregroundColor(.white.opacity(0.9))
                .padding(.bottom, 32)

            Button(action: {}) {
                Text("Shop Now")
                    .font(.title3)
                    .fontWeight(.semibold)
                    .foregroundColor(.white)
                    .padding(.horizontal, 32)
                    .padding(.vertical, 16)
                    .background(Color(red: 0.914, green: 0.271, blue: 0.377))
                    .cornerRadius(8)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 96)
        .background(
            LinearGradient(
                gradient: Gradient(colors: [
                    Color(red: 0.102, green: 0.102, blue: 0.18),
                    Color(red: 0.086, green: 0.129, blue: 0.243)
                ]),
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
    }
}

private struct ProductsView: View {
    var body: some View {
        LazyVGrid(
            columns: [
                GridItem(.adaptive(minimum: 300), spacing: 32)
            ],
            spacing: 32
        ) {
            ProductCardView(
                imageName: "headphones.jpg",
                altText: "Wireless Headphones",
                title: "Wireless Headphones",
                price: "$99.99",
                description: "High-quality wireless audio with 30-hour battery life."
            )
            ProductCardView(
                imageName: "speaker.jpg",
                altText: "Bluetooth Speaker",
                title: "Bluetooth Speaker",
                price: "$49.99",
                description: "Portable speaker with rich bass and 360-degree sound."
            )
            ProductCardView(
                imageName: "earbuds.jpg",
                altText: "Wireless Earbuds",
                title: "Wireless Earbuds",
                price: "$79.99",
                description: "Compact earbuds with active noise cancellation."
            )
        }
    }
}

private struct ProductCardView: View {
    let imageName: String
    let altText: String
    let title: String
    let price: String
    let description: String

    var body: some View {
        VStack(spacing: 0) {
            Image(imageName)
                .resizable()
                .scaledToFill()
                .frame(height: 200)
                .clipped()
                .cornerRadius(8)
                .accessibilityLabel(altText)
                .padding(.bottom, 16)

            Text(title)
                .font(.title2)
                .bold()
                .foregroundColor(.primary)
                .padding(.bottom, 8)

            Text(price)
                .font(.title)
                .fontWeight(.bold)
                .foregroundColor(Color(red: 0.914, green: 0.271, blue: 0.377))
                .padding(.bottom, 8)

            Text(description)
                .font(.body)
                .foregroundColor(Color(red: 0.4, green: 0.4, blue: 0.4))
                .padding(.bottom, 16)

            Button(action: {}) {
                Text("Add to Cart")
                    .font(.body)
                    .fontWeight(.semibold)
                    .foregroundColor(.white)
                    .padding(.horizontal, 24)
                    .padding(.vertical, 12)
                    .background(Color(red: 0.102, green: 0.102, blue: 0.18))
                    .cornerRadius(8)
            }
        }
        .padding(32)
        .background(Color.white)
        .cornerRadius(12)
        .shadow(color: Color.black.opacity(0.1), radius: 6, x: 0, y: 4)
    }
}

private struct FooterView: View {
    var body: some View {
        Text("© 2026 ShopStore. All rights reserved.")
            .font(.body)
            .foregroundColor(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 32)
            .background(Color(red: 0.102, green: 0.102, blue: 0.18))
    }
}
