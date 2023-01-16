from manim import *
from numpy import array
from random import choice

NORMAL_FONT = 20

PRIVATE_KEY_PNG = "private_key.png"
PUBLIC_KEY_PNG = "public_key.png"

Text.set_default(color=BLACK)
MathTex.set_default(color=BLACK)
Dot.set_default(color=BLACK)
Rectangle.set_default(color=BLACK)

class Title(Scene):
    def construct(self):
        self.camera.background_color = WHITE           

        title = Text("CRYSTAL-Dilitihum")
        self.play(Create(title))
        self.wait(1.0)

class ValidatedScenario(Scene):
    def construct(self):
        self.camera.background_color = WHITE

        sender = labeled_dude(color=RED, label="Sender")
        recipient = labeled_dude(color=BLUE, label="Recipient")
        party = Group(sender, recipient).arrange(buff=8)
        self.play(FadeIn(sender))
        self.play(FadeIn(recipient))

        challenge_string = random_signature(6);
        message = Text("Incommutable", font_size=NORMAL_FONT)
        signature = Text(f"z = {random_signature(6)}", font_size=NORMAL_FONT)
        challenge = Text(f"c = {challenge_string}", font_size=NORMAL_FONT)
        packet = Group(message, signature, challenge).arrange(DOWN, buff=0.1).next_to(sender, RIGHT)
        self.play(Create(message))
        self.play(Create(signature))
        self.play(Create(challenge))
        self.play(packet.animate.next_to(recipient, LEFT))
        
        calculated_challenge = Text(f"c' = {challenge_string}", color=BLUE, font_size=NORMAL_FONT).next_to(challenge, DOWN)
        validation_label = Text("validated!", font_size=NORMAL_FONT, color=GREEN).next_to(message, UP)
        self.play(Create(calculated_challenge))
        self.play(Create(validation_label))
        self.wait(1.0)

class RejectedScenario(Scene):
    def construct(self):
        self.camera.background_color = WHITE

        sender = labeled_dude(color=RED, label="Sender")
        recipient = labeled_dude(color=BLUE, label="Recipient")
        party = Group(sender, recipient).arrange(buff=8)
        self.play(FadeIn(sender))
        self.play(FadeIn(recipient))

        challenge_string = random_signature(6);
        message = Text("Incommutable", font_size=NORMAL_FONT)
        signature = Text(f"z = {random_signature(6)}", font_size=NORMAL_FONT)
        challenge = Text(f"c = {challenge_string}", font_size=NORMAL_FONT)
        packet = Group(message, signature, challenge).arrange(DOWN, buff=0.1).next_to(sender, RIGHT)
        self.play(Create(message))
        self.play(Create(signature))
        self.play(Create(challenge))
        self.play(packet.animate.move_to([0, 0, 0]))

        altered_message = Text("Incomputable", font_size=NORMAL_FONT).move_to(message)
        altered_packet = Group(altered_message, signature, challenge)
        self.play(FadeOut(message))
        self.play(FadeIn(altered_message))
        self.play(altered_packet.animate.next_to(recipient, LEFT))
        
        calculated_signature = Text(f"c' = {random_signature(6)}", color=BLUE, font_size=NORMAL_FONT).next_to(challenge, DOWN)
        rejection_label = Text("rejected!", font_size=NORMAL_FONT, color=RED).next_to(altered_message, UP)
        self.play(Create(calculated_signature))
        self.play(Create(rejection_label))

        integrity_label = Text("Integrity")
        authentication_label = Text("Authentication")
        label_group = Group(integrity_label, authentication_label).arrange(DOWN).move_to(3 * UP)
        self.play(Create(integrity_label))

        private_key = key(PRIVATE_KEY_PNG, label="Private key").next_to(sender, DOWN)
        public_key = key(PUBLIC_KEY_PNG, label="Public key").next_to(recipient, DOWN)
        
        self.play(FadeIn(private_key))
        self.play(FadeIn(public_key))
        
        self.play(Create(authentication_label))
        self.wait(1.0)

class Lattice(Scene):
    def construct(self):
        self.camera.background_color = WHITE

        master_equation = MathTex(r"Ay + e = z \text{ mod } p")
        condition = MathTex(r'\text{Find }', 'y', r'\text{ and }', 'e', r'\text{ knowing }', r'A', r'\text{, }', 'z', r'\text{ and }', 'p', r'\text{...}')
        conclusion = Text("...hard to solve!", font_size=NORMAL_FONT)
        label_group = Group(master_equation, condition, conclusion).arrange(DOWN).move_to(3 * UP)
        self.play(Create(master_equation))
        self.play(Create(condition))
        self.play(Create(conclusion))

        bdd_problem = Text("Bounded Distance Decoding problem", font_size=30).next_to(master_equation, DOWN)
        self.play(FadeOut(Group(condition, conclusion)))
        self.play(Create(bdd_problem))

        self.next_section()

        axes = Axes(
            x_range=[-10, 10, 1],
            y_range=[-5, 5, 1],
            x_length=10,
            y_length=5,
            axis_config={"color": GREEN},
            tips=False,
        ).move_to(0.5 * DOWN)
        box = Rectangle(width=axes.width + 0.1, height=axes.height + 0.1).move_to(axes)
        example_lattice = lattice(size=(axes.width, axes.height), dot_radius=0.03, origin=(-7/3, 4/7), step=(3/7, 1/3), shift_step=4/7).move_to(axes)
        closest_element_circle = Circle(radius=0.1).move_to(0.22 * RIGHT + 0.55 * DOWN)
        self.play(Create(box))
        self.play(Create(axes))
        self.play(FadeIn(example_lattice))
        self.play(Create(closest_element_circle))

        pratical_equation = MathTex(r"A", r"s_1", r"+", r"s_2", r"=", r"t", r"\text{ mod } q").move_to(master_equation)
        self.play(AnimationGroup(FadeOut(master_equation), FadeIn(pratical_equation)))
        self.play(AnimationGroup(
            pratical_equation[0].animate.set_color(GREEN),
            pratical_equation[5].animate.set_color(GREEN),
        ))
        self.play(AnimationGroup(
            pratical_equation[1].animate.set_color(RED),
            pratical_equation[3].animate.set_color(RED),
        ))        
        self.wait(1.0)


def lattice(size, dot_radius, origin, step, shift_step):
    retval = []
    
    (x_origin, y_origin) = origin
    (x_step, y_step) = step
    (width, height) = size
    row_length = 2 * max(int((abs(x_origin) + width) / x_step), int((abs(2 * y_origin) + height) / y_step))
    shift_length = 2 * int((height + width) / shift_step) # Probably not the right formula...

    for shift in range(-shift_length // 2, shift_length // 2):
        for step in range(-row_length // 2, row_length // 2 + 1):
            x = step * x_step + x_origin
            y = step * y_step + shift * shift_step + y_origin
            if (-width <= 2 * x <= width and -height <= 2 * y <= height):
                retval.append(Dot(radius=dot_radius).move_to(x * LEFT + y * UP))

    return Group(*retval)
    

def key(file_path, label):
    SIZE = 0.5
    key = ImageMobject(file_path).scale_to_fit_width(SIZE)
    label = Text(label, font_size=NORMAL_FONT).next_to(key, RIGHT)
    return Group(key, label)


def random_signature(length):
    CHARSET = "azertyuiopqsdfghjklmwxcvbnAZERTYUIOPQSDFGHJKLMWXCVBN0123456789"
    return ''.join([choice(CHARSET) for _ in range(length)])

def dude(color):
    HEAD_OFFSET = 0.7 * UP
    BODY_OFFSET = 0.3 * DOWN
    MASK_OFFSET = 0.3 * DOWN

    HEAD_RADIUS = 0.3
    BODY_WIDTH = 1
    BODY_HEIGHT = 2

    FILL_OPACITY = 0.3

    def head(position):
        return Circle(radius=HEAD_RADIUS, color=color, fill_opacity=FILL_OPACITY).move_to(position)

    def body(position, head_obj):
        circle = Ellipse(width=BODY_WIDTH, height=BODY_HEIGHT).move_to(position)
        mask = Square().move_to(circle.point_at_angle(3 * PI / 2)).shift(MASK_OFFSET)
        return Difference(Difference(circle, mask), head_obj, color=color, fill_opacity=FILL_OPACITY)

    head_obj = head(HEAD_OFFSET);
    return Group(head_obj, body(BODY_OFFSET, head_obj))

def labeled_dude(label, **kwargs):
    dude_obj = dude(**kwargs)
    dude_label = Text(label).next_to(dude_obj, UP)
    return Group(dude_obj, dude_label)
