import 'package:flutter/material.dart';

class GeneratedView extends StatelessWidget {
  const GeneratedView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFFF5F5F5),
      body: SafeArea(
        child: Column(
          children: [
            const _Header(),
            Expanded(
              child: SingleChildScrollView(
                child: Column(
                  children: [
                    Center(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: ConstrainedBox(
                          constraints: const BoxConstraints(maxWidth: 800),
                          child: Column(
                            children: const [
                              SizedBox(height: 32),
                              _BlogCard(
                                title: 'Getting Started with TypeScript',
                                meta: 'By Jane Doe \u00b7 5 min read',
                                description:
                                    'TypeScript adds static typing to JavaScript, making your code more predictable and easier to debug. In this post, we explore the basics of TypeScript and how to set up your first project.',
                              ),
                              SizedBox(height: 24),
                              _BlogCard(
                                title: 'CSS Grid Layout Guide',
                                meta: 'By John Smith \u00b7 8 min read',
                                description:
                                    'CSS Grid Layout is a powerful tool for creating complex web layouts. Learn how to use grid-template-areas, fractions, and gaps to build responsive designs.',
                              ),
                              SizedBox(height: 24),
                              _BlogCard(
                                title: 'React 19 New Features',
                                meta: 'By Jane Doe \u00b7 6 min read',
                                description:
                                    'React 19 introduces server components, improved suspense, and new hooks that simplify data fetching. Discover what\'s new and how to upgrade your applications.',
                              ),
                              SizedBox(height: 32),
                            ],
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(height: 48),
                    const _Footer(),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
      decoration: BoxDecoration(
        color: Colors.white,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 3,
            offset: const Offset(0, 1),
          ),
        ],
      ),
      child: Row(
        children: [
          const Text(
            'My Blog',
            style: TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.bold,
              color: Color(0xFF333333),
            ),
          ),
          const Spacer(),
          const _TabLink(label: 'Latest'),
          const SizedBox(width: 24),
          const _TabLink(label: 'Popular'),
          const SizedBox(width: 24),
          const _TabLink(label: 'Archive'),
        ],
      ),
    );
  }
}

class _TabLink extends StatelessWidget {
  final String label;

  const _TabLink({required this.label});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8),
      decoration: const BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Colors.transparent, width: 2),
        ),
      ),
      child: Text(
        label,
        style: const TextStyle(
          fontSize: 16,
          color: Color(0xFF666666),
        ),
      ),
    );
  }
}

class _BlogCard extends StatelessWidget {
  final String title;
  final String meta;
  final String description;

  const _BlogCard({
    required this.title,
    required this.meta,
    required this.description,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.08),
            blurRadius: 8,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(
              fontSize: 28,
              fontWeight: FontWeight.bold,
              color: Color(0xFF1A1A2E),
            ),
          ),
          const SizedBox(height: 8),
          Text(
            meta,
            style: const TextStyle(
              fontSize: 14,
              color: Color(0xFF888888),
            ),
          ),
          const SizedBox(height: 16),
          Text(
            description,
            style: const TextStyle(
              fontSize: 16,
              color: Color(0xFF555555),
            ),
          ),
          const SizedBox(height: 16),
          const Text(
            'Read More',
            style: TextStyle(
              fontSize: 16,
              fontWeight: FontWeight.w600,
              color: Color(0xFF2563EB),
            ),
          ),
        ],
      ),
    );
  }
}

class _Footer extends StatelessWidget {
  const _Footer();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(vertical: 32),
      color: const Color(0xFF1A1A2E),
      child: const Text(
        '\u00a9 2026 My Blog. All rights reserved.',
        textAlign: TextAlign.center,
        style: TextStyle(
          fontSize: 16,
          color: Colors.white,
        ),
      ),
    );
  }
}
