import SwiftUI

struct GeneratedView: View {
    var body: some View {
        ZStack {
            Color(red: 0.961, green: 0.961, blue: 0.961)
                .ignoresSafeArea()

            ScrollView {
                VStack(spacing: 0) {
                    HeaderView()
                        .padding(.bottom, 32)

                    VStack(spacing: 24) {
                        BlogCardView(
                            title: "Getting Started with TypeScript",
                            meta: "By Jane Doe · 5 min read",
                            description: "TypeScript adds static typing to JavaScript, making your code more predictable and easier to debug. In this post, we explore the basics of TypeScript and how to set up your first project.",
                            linkText: "Read More"
                        )
                        BlogCardView(
                            title: "CSS Grid Layout Guide",
                            meta: "By John Smith · 8 min read",
                            description: "CSS Grid Layout is a powerful tool for creating complex web layouts. Learn how to use grid-template-areas, fractions, and gaps to build responsive designs.",
                            linkText: "Read More"
                        )
                        BlogCardView(
                            title: "React 19 New Features",
                            meta: "By Jane Doe · 6 min read",
                            description: "React 19 introduces server components, improved suspense, and new hooks that simplify data fetching. Discover what's new and how to upgrade your applications.",
                            linkText: "Read More"
                        )
                    }
                    .padding(.horizontal, 16)
                    .frame(maxWidth: 800)

                    FooterView()
                        .padding(.top, 48)
                }
                .frame(maxWidth: .infinity)
            }
        }
    }
}

private struct HeaderView: View {
    var body: some View {
        HStack {
            Text("My Blog")
                .font(.title)
                .bold()
                .foregroundColor(Color(red: 0.2, green: 0.2, blue: 0.2))

            Spacer()

            HStack(spacing: 24) {
                Text("Latest")
                    .font(.body)
                    .foregroundColor(Color(red: 0.4, green: 0.4, blue: 0.4))
                Text("Popular")
                    .font(.body)
                    .foregroundColor(Color(red: 0.4, green: 0.4, blue: 0.4))
                Text("Archive")
                    .font(.body)
                    .foregroundColor(Color(red: 0.4, green: 0.4, blue: 0.4))
            }
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 16)
        .background(Color.white)
        .shadow(color: Color.black.opacity(0.1), radius: 3, x: 0, y: 1)
    }
}

private struct BlogCardView: View {
    let title: String
    let meta: String
    let description: String
    let linkText: String

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(title)
                .font(.title2)
                .bold()
                .foregroundColor(Color(red: 0.102, green: 0.102, blue: 0.18))
                .padding(.bottom, 8)

            Text(meta)
                .font(.caption)
                .foregroundColor(Color(red: 0.533, green: 0.533, blue: 0.533))
                .padding(.bottom, 16)

            Text(description)
                .font(.body)
                .foregroundColor(Color(red: 0.333, green: 0.333, blue: 0.333))
                .padding(.bottom, 16)

            Text(linkText)
                .font(.body)
                .fontWeight(.semibold)
                .foregroundColor(Color(red: 0.149, green: 0.388, blue: 0.922))
        }
        .padding(32)
        .background(Color.white)
        .cornerRadius(12)
        .shadow(color: Color.black.opacity(0.08), radius: 8, x: 0, y: 2)
    }
}

private struct FooterView: View {
    var body: some View {
        Text("© 2026 My Blog. All rights reserved.")
            .font(.body)
            .foregroundColor(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 32)
            .background(Color(red: 0.102, green: 0.102, blue: 0.18))
    }
}
